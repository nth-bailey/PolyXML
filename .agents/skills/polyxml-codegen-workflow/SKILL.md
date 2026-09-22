---
name: polyxml-codegen-workflow
description: >-
  Use this skill when developing, refactoring, or testing code generation across PolyXML's 7 target languages
  (Rust, Python, C++, Java, TypeScript, Go, C#), adding codegen backends, plumbing CLI flags and polyxml.toml manifest options,
  or writing compiler integration tests.
---

# PolyXML Polyglot Codegen Development & Verification Playbook

This skill outlines the architectural patterns, options plumbing, and test workflows for the schema compiler and code generators in `crates/polyxml-core/src/codegen/` and `crates/polyxml-cli`.

---

## 1. Codegen Architecture & Layout

All code generators receive a normalized `SchemaIR` from `crates/polyxml-core/src/schema/mod.rs` and generate self-contained source files.

```
crates/polyxml-core/src/codegen/
├── mod.rs                  # Module exports and shared codegen traits
├── rust/mod.rs             # Rust (Zero-copy Cow<'a, str>, Owned, rkyv, streaming codecs)
├── python/mod.rs           # Python 3.12+ (Dataclasses, Pydantic v2, codecs)
├── cpp/mod.rs              # C++20 (Header-only / C++20 Modules, Glaze reflection)
├── java/mod.rs             # Java 21+ (Records, Jackson XML/JSON, sealed interfaces)
├── typescript/mod.rs       # TypeScript 5+ (Interfaces, Zod, Valibot, TypeBox)
├── go/mod.rs               # Go 1.22+ (Structs, xml/json tags, EasyJSON, Sonic)
└── csharp/mod.rs           # C# 12 / .NET 8+ (Record classes/structs, source gen context)
```

---

## 2. Standard Pattern for Codegen Options & Backends

When adding or extending features for a target language:

### 1. Define Backend / Mode Enums with `from_str_loose`
Ensure loose string parsing supports lowercase, hyphens, underscores, and aliases:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetBackend {
    #[default]
    Standard,
    Alternative,
}

impl TargetBackend {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().replace('-', "_").as_str() {
            "alt" | "alternative" => Self::Alternative,
            _ => Self::Standard,
        }
    }
}
```

### 2. Add Fields to `Options` Struct
```rust
#[derive(Debug, Clone)]
pub struct TargetOptions {
    pub backend: TargetBackend,
    pub custom_flag: bool,
}
```

### 3. Re-export in `crates/polyxml-core/src/codegen/mod.rs`
Always re-export the new enum and options from `polyxml_core::codegen` so `polyxml-cli` and consumers can access them cleanly.

---

## 3. Plumbing into CLI and Workspace Manifest (`polyxml-cli`)

New options must be available in **both** single-schema CLI invocations and declarative `polyxml.toml` builds:

1. **Workspace Manifest (`crates/polyxml-cli/src/config.rs`)**:
   - Add fields to `TargetConfig` and `CodegenTargetConfig`.
   - Update `to_emit_options(...)` to forward the values.
2. **CLI Args (`crates/polyxml-cli/src/main.rs`)**:
   - Add flags to `GenerateArgs` (e.g. `--my-flag`, `--backend <NAME>`).
   - Plumb into `TargetEmitOptions` and `handle_generate(...)`.
   - Plumb into `emit_target_code(...)` for multi-target builds in `run_build(...)`.

---

## 4. Testing & Verification Workflows

### Fast Codegen Unit Tests (Sub-second execution)
Instead of running all 23 workspace test suites, run targeted codegen tests while iterating:

```bash
# Run all codegen tests across all languages in ~1 second
./scripts/test_codegen.sh

# Or run language-specific codegen test suites:
cargo test -p polyxml --test test_rust_codegen
cargo test -p polyxml --test test_python_codegen
cargo test -p polyxml --test test_cpp_codegen
cargo test -p polyxml --test test_java_codegen
cargo test -p polyxml --test test_ts_codegen
cargo test -p polyxml --test test_go_codegen
cargo test -p polyxml --test test_csharp_codegen
```

### CLI Integration Tests
Verify CLI flags and `polyxml.toml` manifest execution:

```bash
cargo test -p polyxml-cli --test test_cli
```

### Real Toolchain E2E Validation
If external compilers are installed on the development machine, run the E2E verification tests:

- **.NET 8**: `dotnet build` executes inside `test_cli_csharp_generation`.
- **Go 1.22+**: `go test` executes inside `test_cli_go_generation`.
- **TypeScript**: `tsc --noEmit` runs inside `test_cli_typescript_generation`.
- **C++20**: `g++ -std=c++20` runs inside `test_cli_cpp_generation`.

---

## 5. Invariants & Common Gotchas

1. **Tarjan SCC Cycle-Cutting**: Recursive and self-referencing types must always be wrapped or boxed (`Box<T>`, `std::unique_ptr`, `v.lazy(...)`, `Type.Recursive(...)`, `z.lazy(...)`).
2. **Preserve Defaults**: When `--backend` or new flags are omitted, code generation MUST remain 100% backward compatible with zero behavioral changes.
3. **Closing Braces**: When dynamically emitting interfaces/structs with varying field annotations, ensure trailing braces and blank lines (`out.push_str("}\n\n")`) are consistently emitted.
4. **Git Pre-Push Hook**: Pre-push verifies all package versions match across 5 files. If remote `main` has advanced due to automated CI releases (`chore(release): X.Y.Z`), run `git pull --rebase origin main` before pushing.

## 6. Java/C# Model Styles and Direct Java Codecs

- `--style record|pojo|class` is shared by Java and C#. Records remain the default;
  `pojo`/`class` select mutable models. Java `--builder` and
  `--codec annotation|direct` must also be forwarded in both manifest forms.
- Java mutable models/builders live in `java/models.rs`; StAX companions live in
  `java/codec.rs`. Inheritance must share field-name allocation between accessors,
  builders, and codecs. A derived builder extends its base builder and overrides
  inherited fluent methods with a covariant return type.
- Call `validate_direct_codecs` before CLI emission. Wildcards/dynamic `anyType`
  cannot be silently dropped. The direct reader must consume exactly one element
  and leave the cursor on END_ELEMENT; nested codecs rely on this contract.
- C# mutable style must apply to structs, simple wrappers, union branches, and
  root wrappers. Preserve XML/JSON attributes and invoke inherited validators.
- Real Java compilation/round trips run in `test_java_codegen`; C# mutable XML and
  source-generated JSON round trips run in `test_csharp_codegen`. Jackson/JAXB
  interoperability is tested by `mvn -f benchmarks/java/pom.xml clean test` after
  building the CLI. This requires Maven network access for dependencies initially.
- `benchmarks/java` contains JMH read/write and mutation workloads. Use JDK 22+
  and `-Ppanama` for native comparisons. Newer JDKs require the explicit JMH
  annotation-processor path or the jar lacks `META-INF/BenchmarkList`. Clean the
  Maven target when switching profiles. See its README for workload limits;
  synthetic scalar projections are not full ISO 20022/UCI schema benchmarks.
- CLI help assertions must track `Cli`'s current `about` text. A stale assertion
  expecting “Polyglot XML schema compiler” predates the current help description.
- The sibling checkout `../polyxml-finance-examples` carries the real ISO 20022
  `schemas/finance/pacs_008_core.xsd`. Its `scripts/generate_all.sh` regenerates
  all 7 targets from whichever local `target/{debug,release}/polyxml` is newer;
  a clean `git status` there proves byte-for-byte output parity after codegen
  changes. The same schema is the best large-schema smoke for
  `--style pojo --builder --codec direct`: `javac` the output and round-trip
  `data/pacs_008_customer_credit_transfer.xml` through the generated root codec.
- `benchmarks/java -Ppanama` needs JDK 22+, but the host default can be JDK 21.
  Set `JAVA_HOME`/`PATH` to a downloaded JDK (Temurin 25 worked) and run
  `cargo build --release -p polyxml-c` first so `-Djava.library.path=target/release`
  resolves `libpolyxml.so`. `mvn clean` deletes `target/*.json`, so always rerun
  the JMH smoke (`-p batchSize=10 -wi 0 -i 1 -r 100ms -f 1 -foe true`) after a
  clean build; its JSON lands in the gitignored `benchmarks/java/target/`.
