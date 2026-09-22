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
5. **Never Derive Type Identifiers from `qname.local` Directly**: all named-type
   identifiers must flow through the language's `type_ident` helper (see
   section 7) so cross-namespace collisions stay disambiguated.

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

## 7. Cross-Namespace Type-Name Disambiguation (issue #51 item 3)

Flat (single-file/module) output must survive two types that share a local name
across namespaces (e.g. `urn:a|Address` + `urn:b|Address`). The mechanism lives
in `crates/polyxml-core/src/codegen/mod.rs`:

- `build_type_name_map(ir, name_of)` assigns a unique identifier to every type
  in `ir.types`: the first name in `BTreeMap` iteration order keeps the bare
  name; collisions get numeric suffixes (`Address`, `Address2`, `Address3`, …).
- The map is installed per thread via `set_type_name_map(...)` into a
  `thread_local!` `RefCell<HashMap<QName, String>>` and read with
  `lookup_type_name(qname, fallback)`. Names absent from the map (references to
  unloaded external types) fall back to the language's default derivation —
  single-namespace schemas are therefore byte-for-byte unchanged.

Rules when touching any codegen:

1. **Install the map at every public entry that emits types.** For most
   languages that is `generate_module`; Java also needs it in `generate_files`
   (it calls `generate_single_type` directly) and C++ needs it in both
   `generate_header` and `generate_module_unit` (both are real emitters).
2. **Each language defines one `type_ident(q: &QName)` helper** whose fallback
   equals the language's historical derivation exactly (`AsPascalCase` for
   Rust/Python, `to_java_type_name`, `to_cpp_type_name`, `to_csharp_type_name`,
   `to_ts_type_name`, `to_go_type_name`). Route every *identifier* site through
   it: declarations, `TypeRef::Named` mappings, base/inheritance clauses,
   `typeof`/`extends`/`Schema` references, codec class names, and the
   `declared_names` sets used to guard root-element aliases.
3. **Never disambiguate wire names.** XML tags, `xml_name`, `[XmlRoot(...)]`,
   Jackson's `localName`, Pydantic `Meta.name`, `variant_name`s, and root
   aliases derived from *element* names keep using raw local names.
4. Special-shaped sites must preserve historical output when nothing
   collides: C++ `{enum}_from_string` uses the snake-of-local stem unless the
   disambiguated name differs; TypeScript `{name}Schema` refs swap the inner
   name only (declarations derive from the same `type_ident`).
5. Regression lock: `issue51_item3_*` in
   `crates/polyxml-core/tests/test_issue51_audit.rs` asserts both colliding
   types surface as `Address` + `Address2` in all 7 languages.

## 8. Python Multi-Pattern Validation (issue #51 item 8)

Pydantic's `Field(pattern=...)` accepts a single regex, and multiple
`StringConstraints(pattern=...)` annotations do NOT AND (the last one wins).
When a simple type carries **more than one** pattern facet (common since
derived types inherit their base's patterns at parse time):

- `emit_simple_type` appends `AfterValidator(_polyxml_patterns(r"p1", r"p2"))`
  to the `Annotated[...]` alias (single-pattern types keep the plain
  `Field(pattern=...)` kwarg, byte-identical to before).
- `needs_pattern_validator(ir)` gates emission of the module-level
  `_polyxml_patterns` factory helper, the `import re` line, and the
  `AfterValidator,` prefix on the pydantic import — keep these three in sync.
- Semantics: unanchored `regex.search` per pattern, all must match (AND),
  matching XSD pattern search semantics.
- Field-level facets (`FieldDef.facets`) are never populated by the parser
  today, so `emit_struct`'s field path needs no `AfterValidator` wiring; if
  that ever changes, mirror the type-level handling there.
