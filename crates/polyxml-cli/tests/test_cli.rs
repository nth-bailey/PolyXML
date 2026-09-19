use std::fs;
use std::process::Command;
use tempfile::tempdir;

use polyxml_cli::config::WorkspaceManifest;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Polyglot XML schema compiler"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_cli_generate_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["generate", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--lang"));
    assert!(stdout.contains("--out"));
    assert!(stdout.contains("--strict-facets"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_cli_validate_valid_and_invalid() {
    let dir = tempdir().unwrap();
    let valid_xsd = dir.path().join("test.xsd");
    fs::write(
        &valid_xsd,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:test">
            <xs:element name="Hello" type="xs:string"/>
        </xs:schema>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["validate", valid_xsd.to_str().unwrap()])
        .output()
        .expect("Failed to execute validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Valid schema"));
    assert!(stdout.contains("urn:test"));

    // Test non-existent schema
    let missing_xsd = dir.path().join("missing.xsd");
    let bad_output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["validate", missing_xsd.to_str().unwrap()])
        .output()
        .expect("Failed to execute validate");

    assert!(!bad_output.status.success());
}

#[test]
fn test_cli_generate_dry_run() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("order.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:orders">
            <xs:complexType name="Order">
                <xs:sequence>
                    <xs:element name="Id" type="xs:string"/>
                    <xs:element name="Amount" type="xs:decimal"/>
                </xs:sequence>
            </xs:complexType>
            <xs:element name="PurchaseOrder" type="Order"/>
        </xs:schema>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["generate", "--dry-run", schema_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute generate --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run completed successfully. No files written."));
    assert!(stdout.contains("Namespace: urn:orders"));
    assert!(stdout.contains("Root elements: 1"));
}

#[test]
fn test_cli_generate_multi_lang() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("sample.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:sample">
            <xs:complexType name="SampleItem">
                <xs:sequence>
                    <xs:element name="Name" type="xs:string"/>
                </xs:sequence>
            </xs:complexType>
        </xs:schema>"#,
    )
    .unwrap();

    let out_dir = dir.path().join("out");

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "python",
            "--lang",
            "rust",
            "--out",
            out_dir.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute generate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Code generation complete."));

    assert!(out_dir.join("python/sample.py").exists());
    assert!(out_dir.join("rust/sample.rs").exists());
}

#[test]
fn test_polyxml_toml_manifest_build() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("contract.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:corp">
            <xs:element name="Agreement" type="xs:string"/>
        </xs:schema>"#,
    )
    .unwrap();

    let manifest_file = dir.path().join("polyxml.toml");
    let manifest_content = r#"
[workspace]
name = "contracts"
schemas = ["*.xsd"]
output_base_dir = "./dist"

[[generate]]
target = "python"
output = "py_models"
backend = "pydantic-v2"

[codegen.rust]
enabled = true
output = "rs_models"
zero_copy = true
"#;
    fs::write(&manifest_file, manifest_content).unwrap();

    // Test config parsing unit
    let parsed = WorkspaceManifest::from_file(&manifest_file).unwrap();
    assert_eq!(
        parsed.workspace.as_ref().unwrap().name.as_deref(),
        Some("contracts")
    );
    let targets = parsed.resolved_targets();
    assert_eq!(targets.len(), 2);
    let schemas = parsed.expand_schemas(dir.path()).unwrap();
    assert_eq!(schemas.len(), 1);

    // Test CLI build command
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["build", "--config", manifest_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute build");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build finished successfully."));
    assert!(dir.path().join("dist/py_models/contract.py").exists());
    assert!(dir.path().join("dist/rs_models/contract.rs").exists());
}
