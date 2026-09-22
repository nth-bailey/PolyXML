---
name: polyxml-core-engine
description: >-
  Use this skill when developing, refactoring, or optimizing the pure Rust core XML engine in crates/polyxml-core,
  streaming quick-xml reader/writer events, using lexical-core for conversions, or designing schema IRs.
---

# PolyXML Pure Rust Core Engine Architecture & Patterns

This skill documents the high-performance design patterns and strict constraints for
`crates/polyxml-core`.

## 1. Core Engine Invariants

1. **Zero Python Dependencies**:
   - `crates/polyxml-core` must remain 100% pure, idiomatic Rust.
   - Never import `pyo3`, Python runtime types, or Python C-API dependencies in this crate.
2. **Streaming Event Model**:
   - Avoid building DOM trees in memory during parsing.
   - Stream tokens using `quick-xml::events::Event` (`Event::Start`, `Event::End`, `Event::Text`, `Event::Empty`).
3. **Zero Intermediate Allocations**:
   - Parse integers, floats, and booleans directly from raw byte slices `&[u8]` using `lexical_core::parse` and custom byte parsers.
   - Use `smallvec::SmallVec` for small, bounded collections of attributes or namespace declarations on the stack.

## 2. Testing & Verification

Run tests and clippy specifically against `polyxml-core` (package name `polyxml`):

```bash
# Run core crate unit tests
cargo test -p polyxml

# Check formatting
cargo fmt --check

# Strict warnings-as-errors clippy audit
cargo clippy -p polyxml --all-targets -- -D warnings
```

## 3. Benchmarking Core Engine

To run benchmarks on the pure Rust engine against XML payloads:

```bash
cargo bench -p polyxml
```

## 4. Schema Parser Architecture (`schema_parser/mod.rs`, post issue #51)

The XSD parser runs **frame-based post-passes**: `parse_str_internal` wraps
`parse_str_body` with a frame-depth counter, and every frame end (schema root,
include/import) runs `inherit_pattern_facets` → `expand_group_refs` →
`resolve_cycles` (last, so cuts see the final graph).

Key mechanisms (regression-locked in `crates/polyxml-core/tests/test_issue51_audit.rs`):

1. **Named groups**: `<xs:group name>` bodies are captured into
   `GroupDef`/`PendingGroupRef` state; `<xs:group ref>` usages expand via
   `parse_group_body` + `consume_inline_element_type` at frame end.
2. **Enum dedup**: duplicate `<xs:enumeration>` values are dropped; distinct
   values whose identifiers collide get numeric suffixes.
3. **Nested inline types**: an `<xs:element>` carrying an inline
   `complexType`/`simpleType` is extracted as a uniquely named top-level type
   (`unique_type_name`) — its fields must never leak into the parent struct.
4. **simpleContent**: `<xs:extension>` emits a `Text` field named `value`
   carrying the extension base type.
5. **Pattern inheritance**: derived simple types append their base type's
   patterns (AND semantics) via `inherit_pattern_facets`; enum facet
   inheritance is intentionally skipped (`EnumDef` has no facets).
6. **File cache & chameleon includes**: the cache stores the raw post-passed
   IR keyed by canonical path (`file_cache`), replays group state
   (`file_groups`) on hit, and re-keys chameleon (namespace-less) includes
   per-includer via `rekey_to_namespace`/`rekey_new_state`.
7. Parser reuse across `parse_file` calls is supported; groups partially
   replay from `file_groups` on cache hit.

## 5. `xsi:type` Polymorphic Dispatch (issue #53)

The runtime dispatches polymorphic elements through a **type registry on
`ModelSchema`** — never through generated code. Documented for users in
`docs/guides/polymorphism.md`; regression-locked in
`crates/polyxml-core/tests/test_issue53_xsi_type.rs`.

- **Registry**: `ModelSchema` carries `is_abstract: bool` plus a
  `variants: OnceLock<Vec<Arc<ModelSchema>>>` (methods: `set_variants`,
  `variants`, `has_variants`, `find_variant(local_bytes)`,
  `matches_variant(candidate)`). `OnceLock` keeps `ModelSchema`'s `Clone`
  derive intact and guarantees each schema registers once. Variants are
  keyed by the type QName local part (`xml_name`), which is `Meta.name` on
  the Python side and the type local on the XSD side.
- **Population (`schema.rs::from_ir`)**: `build_struct` (a) flattens the
  `xs:extension` content model by collecting the base-chain structs
  (root-base first, cycle-guarded) and prepending their fields — mirroring
  Java codegen's `collect`, so derived runtime schemas carry inherited
  fields — and (b) scans the `BTreeMap` of types for transitive
  derivations (`derives_from`), building each inside a `visited`
  insert/remove guard before `set_variants`. **Python-side population
  (pyo3 `lib.rs`)**: after both global caches are inserted in
  `get_or_create_schema_meta`, `discover_variants` walks
  `__subclasses__()` transitively (ptr-seen set, extracts only
  dataclass/Pydantic classes) — running *after* cache insertion so a
  subclass field typed as the base resolves from cache instead of
  re-entering extraction.
- **Parse (`parser.rs`)**: `resolve_record_schema(declared, start)` runs at
  *every* frame creation site — root `Start`/`Empty` in
  `deserialize_with_limit`, the root frame in `parse_sub_tree`, both
  nested `Start`/`Empty` arms (single and list), and the `XmlItemStream`
  `Empty` arm. `xsi_type_value` matches the attribute **local name**
  `type` (namespace-blind, like `is_nil_element`). Policy: registered
  variant → switch schema; unknown value on `is_abstract` →
  `PolyXmlError::SchemaError` listing known variants, or an
  "escape hatch" error when none are registered; everything else falls
  back to the declared schema. Schemas that are neither abstract nor have
  variants return immediately without scanning attributes (zero-overhead
  fast path — this is also why a content attribute named `type` can never
  be misdispatched on plain types).
- **Serialize (`serializer.rs`)**: `write_model` checks
  `schema.matches_variant(rec)` on `Record` values, shadows the effective
  schema to the variant, and pushes `xsi:type` (qualified QName via
  `qualify_element`, falling back to literal `xsi:type` when namespaces
  are disabled) after content attributes. `NamespaceContext::collect_namespaces`
  recurses `variants` and injects the XSI URI so `xmlns:xsi` is always
  declared at the root whenever dispatch is possible.
- **JSON (`json.rs`)**: `poly_value_to_json_value` prefers the record's
  own schema when its name differs from the declared one, so variant-only
  fields survive `xml_to_json` (JSON carries no selector marker).
- **Escape hatches**: generated static codecs (Rust serde, Java) do *not*
  dispatch — Java's codec errors loudly; deserialize the concrete type or
  use the dynamic runtime. `polyxml-js` bindings are scalar-flat today, so
  dispatch is unreachable from JS until nested schemas land (shared core
  means no future binding work).
