pub mod config;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use clap::{Args, Parser, Subcommand};
use config::WorkspaceManifest;
use polyxml::codegen::python::{PythonBackend, PythonCodegen, PythonOptions};
use polyxml::codegen::rust::{RustCodegen, RustOptions};
use polyxml::ir::{SchemaIR, TypeDef};
use polyxml::schema_parser::XsdParser;

#[derive(Debug, Parser)]
#[command(
    name = "polyxml",
    about = "Polyglot XML schema compiler and data-binding toolchain",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Compile XSD schema(s) into target language models
    Generate(GenerateArgs),

    /// Orchestrate multi-language project generation from polyxml.toml
    Build(BuildArgs),

    /// Validate XML schema syntax and structural invariants without generating code
    Validate(ValidateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Path(s) to XSD schema files or glob patterns
    #[arg(value_name = "SCHEMA")]
    pub schemas: Vec<PathBuf>,

    /// Target language(s) to emit (python, rust, cpp, java, ts, go, csharp)
    #[arg(short = 'l', long = "lang", value_name = "LANG")]
    pub lang: Vec<String>,

    /// Target language backend (e.g. 'dataclass' or 'pydantic' for python)
    #[arg(short = 'b', long = "backend", value_name = "BACKEND")]
    pub backend: Option<String>,

    /// Zero-copy mode for Rust models (borrow Cow<'a, str> instead of owned String)
    #[arg(long = "zero-copy", default_missing_value = "true", num_args = 0..=1)]
    pub zero_copy: Option<bool>,

    /// Output directory for generated source files
    #[arg(short = 'o', long = "out", alias = "out-dir", value_name = "DIR")]
    pub out: Option<PathBuf>,

    /// Path to workspace manifest (defaults to ./polyxml.toml if present)
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Enforce restriction facet validators in generated code
    #[arg(long = "strict-facets")]
    pub strict_facets: bool,

    /// Parse and validate schema without writing output files
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Automatically run language-specific code formatters after generation
    #[arg(long = "format")]
    pub format: bool,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to workspace manifest
    #[arg(
        short = 'c',
        long = "config",
        value_name = "FILE",
        default_value = "polyxml.toml"
    )]
    pub config: PathBuf,

    /// Parse and validate without writing files
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Automatically run language-specific formatters
    #[arg(long = "format")]
    pub format: bool,
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Path(s) to XSD schema files to validate
    #[arg(required = true, value_name = "SCHEMA")]
    pub schemas: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate(args) => run_generate(args),
        Commands::Build(args) => run_build(args),
        Commands::Validate(args) => run_validate(args),
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run_generate(args: GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    // If no schemas are passed directly, check for polyxml.toml config
    if args.schemas.is_empty() {
        let config_path = args.config.unwrap_or_else(|| PathBuf::from("polyxml.toml"));
        if config_path.exists() {
            return run_build(BuildArgs {
                config: config_path,
                dry_run: args.dry_run,
                format: args.format,
            });
        } else {
            eprintln!("No schema files specified and polyxml.toml not found.");
            eprintln!("Run 'polyxml generate --help' for usage.");
            process::exit(1);
        }
    }

    let mut parser = XsdParser::new();
    let mut compiled_schemas = Vec::new();

    for schema_path in &args.schemas {
        if !schema_path.exists() {
            return Err(format!("Schema file not found: {}", schema_path.display()).into());
        }

        println!("Parsing schema: {}", schema_path.display());
        let ir = parser.parse_file(schema_path)?;
        report_schema_ir(&ir);
        compiled_schemas.push((schema_path.clone(), ir));
    }

    if args.dry_run {
        println!("\nDry run completed successfully. No files written.");
        return Ok(());
    }

    let languages = if args.lang.is_empty() {
        vec!["python".to_string()]
    } else {
        args.lang
    };

    let base_out = args.out.unwrap_or_else(|| PathBuf::from("generated"));

    for lang in &languages {
        let lang_out = if languages.len() > 1 {
            base_out.join(lang)
        } else {
            base_out.clone()
        };

        fs::create_dir_all(&lang_out)?;
        println!("Generating {} models into {}", lang, lang_out.display());

        for (schema_path, ir) in &compiled_schemas {
            emit_target_code(
                lang,
                args.backend.as_deref(),
                args.zero_copy,
                &lang_out,
                schema_path,
                ir,
            )?;
        }

        if args.format {
            run_language_formatter(lang, &lang_out);
        }
    }

    println!("Code generation complete.");
    Ok(())
}

fn run_build(args: BuildArgs) -> Result<(), Box<dyn std::error::Error>> {
    if !args.config.exists() {
        return Err(format!("Manifest not found: {}", args.config.display()).into());
    }

    println!("Loading manifest: {}", args.config.display());
    let manifest = WorkspaceManifest::from_file(&args.config)?;
    let base_dir = args.config.parent().unwrap_or_else(|| Path::new("."));

    let schema_files = manifest.expand_schemas(base_dir)?;
    if schema_files.is_empty() {
        println!("No schema files matched workspace schema patterns.");
        return Ok(());
    }

    let mut parser = XsdParser::new();
    let mut compiled_schemas = Vec::new();

    for schema_path in &schema_files {
        println!("Compiling schema: {}", schema_path.display());
        let ir = parser.parse_file(schema_path)?;
        report_schema_ir(&ir);
        compiled_schemas.push((schema_path.clone(), ir));
    }

    let targets = manifest.resolved_targets();
    if targets.is_empty() {
        println!("No generation targets configured in manifest.");
        return Ok(());
    }

    if args.dry_run {
        println!("\nDry run completed. Targets configured: {}", targets.len());
        for target in &targets {
            println!(" - Target: {} -> {}", target.target, target.output);
        }
        return Ok(());
    }

    let output_base = manifest
        .workspace
        .as_ref()
        .and_then(|w| w.output_base_dir.as_ref())
        .map(|dir| {
            let p = Path::new(dir);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                base_dir.join(p)
            }
        })
        .unwrap_or_else(|| base_dir.to_path_buf());

    for target in &targets {
        let target_dir = output_base.join(&target.output);
        fs::create_dir_all(&target_dir)?;
        println!(
            "Emitting target [{}] into {}",
            target.target,
            target_dir.display()
        );

        for (schema_path, ir) in &compiled_schemas {
            emit_target_code(
                &target.target,
                target.backend.as_deref(),
                target.zero_copy,
                &target_dir,
                schema_path,
                ir,
            )?;
        }

        if args.format {
            run_language_formatter(&target.target, &target_dir);
        }
    }

    println!("Build finished successfully.");
    Ok(())
}

fn run_validate(args: ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = XsdParser::new();
    let mut total_types = 0;
    let mut total_elements = 0;

    for schema_path in &args.schemas {
        if !schema_path.exists() {
            return Err(format!("Schema file not found: {}", schema_path.display()).into());
        }

        let ir = parser.parse_file(schema_path)?;
        println!("✓ Valid schema: {}", schema_path.display());
        if let Some(ref ns) = ir.target_namespace {
            println!("  targetNamespace: {}", ns);
        }
        println!(
            "  Components: {} types, {} root elements",
            ir.types.len(),
            ir.elements.len()
        );

        total_types += ir.types.len();
        total_elements += ir.elements.len();
    }

    println!(
        "\nAll schemas valid (Total: {} types, {} elements).",
        total_types, total_elements
    );
    Ok(())
}

fn report_schema_ir(ir: &SchemaIR) {
    let mut structs = 0;
    let mut enums = 0;
    let mut unions = 0;
    let mut simples = 0;
    let mut cycle_cuts = 0;

    for type_def in ir.types.values() {
        match type_def {
            TypeDef::Struct(s) => {
                structs += 1;
                for f in &s.fields {
                    if f.is_cycle_cut {
                        cycle_cuts += 1;
                    }
                }
            }
            TypeDef::Enum(_) => enums += 1,
            TypeDef::Union(_) => unions += 1,
            TypeDef::Simple(_) => simples += 1,
        }
    }

    if let Some(ref ns) = ir.target_namespace {
        println!("  Namespace: {}", ns);
    }
    println!(
        "  Types: {} total ({} structs, {} enums, {} unions, {} simple restrictions)",
        ir.types.len(),
        structs,
        enums,
        unions,
        simples
    );
    println!("  Root elements: {}", ir.elements.len());
    if cycle_cuts > 0 {
        println!("  Tarjan SCC: Boxed {} recursive cut points", cycle_cuts);
    }
}

fn emit_target_code(
    lang: &str,
    backend: Option<&str>,
    zero_copy: Option<bool>,
    out_dir: &Path,
    schema_path: &Path,
    ir: &SchemaIR,
) -> std::io::Result<()> {
    match lang.to_lowercase().as_str() {
        "python" | "py" => {
            let py_backend = backend
                .and_then(PythonBackend::from_str_loose)
                .unwrap_or(PythonBackend::Dataclass);

            let options = PythonOptions {
                backend: py_backend,
                slots: true,
                kw_only: true,
                pep695_aliases: true,
                emit_meta: true,
                emit_root_aliases: true,
            };

            let codegen = PythonCodegen::new(options);
            let code = codegen.generate_module(ir);

            let file_stem = schema_path
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_else(|| "models".into());

            let file_path = out_dir.join(format!("{}.py", file_stem));
            fs::write(file_path, code)?;

            let init_path = out_dir.join("__init__.py");
            if !init_path.exists() {
                let _ = fs::write(&init_path, "# Package generated by PolyXML\n");
            }
            Ok(())
        }
        "rust" | "rs" => {
            let options = RustOptions {
                zero_copy: zero_copy.unwrap_or(true),
                derive_serde: true,
                derive_default: true,
                emit_polyxml_attrs: true,
                emit_root_aliases: true,
            };

            let codegen = RustCodegen::new(options);
            let code = codegen.generate_module(ir);

            let file_stem = schema_path
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_else(|| "models".into());

            let file_path = out_dir.join(format!("{}.rs", file_stem));
            fs::write(file_path, code)?;

            let mod_path = out_dir.join("mod.rs");
            if !mod_path.exists() {
                let _ = fs::write(
                    &mod_path,
                    format!("pub mod {};\npub use {}::*;\n", file_stem, file_stem),
                );
            }
            Ok(())
        }
        _ => emit_target_placeholder(lang, out_dir, schema_path, ir),
    }
}

fn emit_target_placeholder(
    lang: &str,
    out_dir: &Path,
    schema_path: &Path,
    ir: &SchemaIR,
) -> std::io::Result<()> {
    let file_stem = schema_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "models".into());

    let (filename, header) = match lang.to_lowercase().as_str() {
        "python" | "py" => (
            format!("{}.py", file_stem),
            "# Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\nfrom __future__ import annotations\n",
        ),
        "rust" | "rs" => (
            format!("{}.rs", file_stem),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\nuse polyxml::ir::*;\n",
        ),
        "cpp" | "c++" => (
            format!("{}.hpp", file_stem),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\n#pragma once\n",
        ),
        "java" => (
            format!("{}.java", heck::AsPascalCase(file_stem.as_ref())),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\n",
        ),
        "ts" | "typescript" => (
            format!("{}.ts", file_stem),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\n",
        ),
        "go" => (
            format!("{}.go", file_stem),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\npackage models\n",
        ),
        "csharp" | "c#" | "cs" => (
            format!("{}.cs", heck::AsPascalCase(file_stem.as_ref())),
            "// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\nnamespace Generated;\n",
        ),
        _ => (
            format!("{}.txt", file_stem),
            "// Generated by PolyXML Compiler\n",
        ),
    };

    let file_path = out_dir.join(filename);
    let mut content = String::from(header);
    content.push_str(&format!(
        "\n// Schema: {}\n// Target Namespace: {}\n// Total Types: {}\n",
        schema_path.display(),
        ir.target_namespace.as_deref().unwrap_or("None"),
        ir.types.len()
    ));

    fs::write(file_path, content)?;
    Ok(())
}

fn run_language_formatter(lang: &str, dir: &Path) {
    let dir_str = match dir.to_str() {
        Some(s) => s,
        None => return,
    };

    match lang.to_lowercase().as_str() {
        "python" | "py" => {
            let _ = Command::new("ruff").args(["format", dir_str]).status();
            let _ = Command::new("ruff")
                .args(["check", "--fix", "--silent", dir_str])
                .status();
        }
        "rust" | "rs" => {
            let _ = Command::new("cargo").args(["fmt"]).status();
        }
        "go" => {
            let _ = Command::new("gofmt").args(["-w", dir_str]).status();
        }
        "cpp" | "c++" => {
            let _ = Command::new("clang-format").args(["-i"]).status();
        }
        _ => {}
    }
}
