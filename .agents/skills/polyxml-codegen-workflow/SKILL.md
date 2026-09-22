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
6. **`xsd:extension` Base-Field Flattening (Rust + Java records)**: targets
   without inheritance must inline the base chain via the shared
   `codegen/mod.rs::flatten_fields(s, ir)` (root first, cycle-cut on revisit,
   most-derived declaration of a schema name shadows inherited duplicates so
   the simpleContent `value` field stays singular). For Rust it is required at
   EVERY field-iteration site: `compute_types_with_lifetime`, `emit_struct`,
   the `has_patterns` probe, and the decode `field_metas` build —
   `encode_xml`/`validate_patterns` follow `field_metas` automatically.
   Missing the lifetime site emits `Cow<'a, str>` in a struct without `<'a>`
   (generated code fails to compile); missing codec sites silently drops
   inherited data. Java's `model_fields` returns the flattened list for every
   mode — records declare it as components, classes re-slice the own-fields
   tail because they `extends` instead. Backends with real inheritance (TS
   `extends`, Go embed, C++/C# `: Base`, Python `class Derived(Base)`) must
   NOT flatten — Python leans on `__dataclass_fields__` (see §11).
7. **Known Gap (found 2026-09-22, unfixed)**: generated Rust codecs never
   consume `FieldKind::Text` — only a `#[polyxml(text)]` label exists — so
   simpleContent structs always fail `from_xml` with
   `Missing required field 'value'` and `encode_xml` omits the text; Go, Java,
   C++, and C# codecs do handle Text. When touching this area, verify with a
   simpleContent XSD across all 7 backends. (The companion Java gap — plain
   record mode dropping inherited fields — was fixed the same day: records
   cannot `extends`, so `model_fields` now always inlines `flatten_fields`.)

## 6. Java/C# Model Styles and Direct Java Codecs

- `--style record|pojo|class` is shared by Java and C#. Records remain the default;
  `pojo`/`class` select mutable models. Java's `--feature builder,direct-codec`
  must also be forwarded as `features = [...]` in both manifest forms.
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
  `--style pojo --feature builder,direct-codec`: `javac` the output and round-trip
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

## 9. Unified CLI Options (issue #50)

- `polyxml-cli/src/options.rs` normalizes and validates backend/style/features
  for direct generation and both manifest forms. Validate all targets before
  schema parsing, dry-run success, or output creation. Core `from_str_loose`
  parsers can still return `None`; the CLI must reject invalid values before
  emitter defaults are applied.
- New opt-ins belong in repeatable `--feature` and manifest `features` arrays.
  The legacy hidden flags (`--zod`, `--source-gen`, `--record-kind`, `--rkyv`,
  `--builder`, `--codec`) and their manifest keys were deleted outright: clap
  rejects the flags as unexpected arguments, `#[serde(deny_unknown_fields)]`
  rejects the keys, and the deprecation-warning loop no longer exists. Keep
  CLI and manifest parity.
- `--zero-copy` (and manifest `zero_copy`) stays a first-class bool because
  `--feature` cannot express `false`; it alone selects owned Rust output, and
  `--zero-copy=false --feature zero-copy` is still rejected as a contradiction.
- Defaults remain unchanged, including C# record classes, Rust zero-copy, and
  Python slots/kw-only. Expose only implemented styles/features. The issue's
  future examples (phf, aot, Python plain class, C# mutable struct) are not yet
  generator capabilities.
- Shared CLI options apply to every `--lang`, not just the preceding one. Use
  per-target manifest entries for heterogeneous configurations. Schema-less
  `generate` must not silently ignore generation overrides.

- Shell completion adapters live under `polyxml-cli/src/completions/`. Their
  hidden `__complete` helper filters candidates through the same option resolver
  used for generation; preserve multi-target intersection and language aliases.
  Bash is exercised in CLI integration tests. Zsh/Fish tests run when those
  shells are available on PATH, and skip otherwise.
- `benchmarks/cli/benchmark.sh` uses hyperfine with `--shell=none` to avoid shell
  calibration error for sub-5ms startup measurements. Results are fresh processes
  with warm OS caches, not machine-reboot or cold-disk startup measurements.

- Color diagnostics respect `NO_COLOR`. When checking terminal color in a PTY,
  unset `NO_COLOR` in the test subprocess and use a color-capable `TERM`; test
  the opt-out separately. This development environment sets `NO_COLOR=1`.

## 10. Pattern OR/AND Semantics & Enforcement (issue #54)

W3C XSD combines `xs:pattern` two ways: multiple patterns **within one
`<xs:restriction>` are OR'd**; patterns inherited **across derivation steps are
AND'd**. The IR keeps `RestrictionFacets.patterns: Vec<String>` flat — the
parser encodes each restriction's alternatives as ONE regex group:

- `schema_parser` collapses `patterns.len() > 1` per restriction into
  `(alt1)|(alt2)` (a singleton stays verbatim), then inheritance prepends/appends
  each derived step's entry. Downstream code just ANDs the flat list — each
  entry is already an OR-group. **Never** AND the pre-collapse alternatives.
- Round-trip tests must serialize `RestrictionFacets`, not the whole
  `SchemaIR`: `SchemaIR.types` is a `BTreeMap<QName, _>` and serde_json rejects
  struct map keys ("key must be a string"). This is pre-existing and unrelated
  to facets.

Shared helpers in `codegen/mod.rs` (use them; don't re-derive):

- `primitive_base(ty, ir)` — resolves a simple alias chain to its primitive,
  cycle-safe. Patterned simple types must emit their **primitive** base (e.g.
  `std::string`, not the alias) so validators/codecs operate on real scalars.
- `patterned_simple(ty, ir)` — the patterned `SimpleTypeDef` behind a field
  type, unwrapping `Boxed`/`List`.

Per-language enforcement (all gated so unpatterned schemas stay byte-identical):

- **Rust**: free `validate_{Type}_patterns(&str)` fns with `OnceLock<Regex>`
  per pattern; structs gain `validate_patterns()` called from `from_xml`/
  `to_xml`, attributes/text/empty-element paths call the free fn directly.
- **Go**: `Validate() error` on patterned aliases (+ `UnmarshalText`/
  `MarshalText` for string bases so `encoding/xml` enforces on read/write);
  struct `Validate` recurses into fields, lists, and pointers. Imports
  `regexp`/`fmt` only when patterns exist.
- **C++**: free `validate_{Type}_patterns(std::string_view)` with function-local
  `static const std::regex`; struct `validate()` calls it. **Inline field
  expressions into the call** — binding `const auto& value = <field>;`
  self-references when the field is named `value` (range-for over a member named
  `value` is fine). `#include <regex>` is conditional on `ir`.
- **Java**: `Pattern.compile(p).matcher(v).find()` (search semantics —
  `Pattern.matches` anchors and is wrong for XSD); direct codecs resolve
  patterned aliases through `primitive_base`.
- **C#**: `Regex.IsMatch` **without** `^...$` anchors (XSD patterns are
  unanchored searches); escape `\` before `"` in the pattern string.
- **TypeScript**: one pattern → `pattern:` option (zod/valibot) or single
  `Type.String({pattern})`; multiple → `AfterValidator`-equivalent AND via
  `_polyxml_patterns` (Python) / `Type.Intersect([...])` (TypeBox — duplicating
  the `pattern:` key silently kept only one). Escape `/` inside regex literals.

Execution tests live in `crates/polyxml-core/tests/test_pattern_codegen.rs`
(wired into `scripts/test_codegen.sh`): they **run** generated Go/C++/Rust/
Python/Java/C# validators against values matching only one alternative, values
satisfying two restriction steps, and reject lists/optionals carrying a bad
element. The TS leg needs `POLYXML_TS_TEST_MODULES` pointing at a node_modules
with `zod@3 valibot@1 @sinclair/typebox@0.34 typescript@5` and skips otherwise.

## 11. Python Abstract Meta & Runtime Type Discovery (issue #53)

`xsi:type` dispatch is a **dynamic-runtime feature** (see
`polyxml-core-engine` skill §5 and `docs/guides/polymorphism.md`); the
codegen only has to expose two facts to the PyO3 layer:

- **`Meta.abstract = True`** is emitted for `is_abstract` structs in
  `codegen/python/mod.rs` (inside the `emit_meta` block, after
  `namespace`). PyO3 reads it in `extract_schema_from_class` →
  `builder.is_abstract(true)`, which turns unknown `xsi:type` values into
  loud errors instead of silent truncation. Only abstract types may emit
  it — locked by `test_python_abstract_meta_emission`.
- **Class inheritance must stay intact**: `emit_struct` already emits
  `class Derived(Base):`, and PyO3 builds a derived class's schema from
  `__dataclass_fields__`/`model_fields`, which include inherited fields —
  do not flatten inheritance away or variant schemas will miss base fields.

Key gotchas when touching this area:

- Variant discovery (`discover_variants`) must run **after** both global
  cache inserts in `get_or_create_schema_meta`, otherwise a subclass field
  typed as the base class re-enters extraction and recurses infinitely.
- Variant keys come from `Meta.name` (the XML type name), not the class
  name — codegen may mangle class identifiers (`type_ident`) while
  `xsi:type` always carries the wire QName local part.
- `polyxml.serialize` takes an optional **`target_type`** (pyo3 + wrapper):
  given the declared base, a concrete instance is converted against the
  base schema and the variant's record triggers `xsi:type` re-emission at
  the root; omitted, the instance's concrete schema is used (no selector).
  Both branches are covered in `tests/test_xsi_type.py` (11 tests, kept at
  100% statement/branch coverage).
