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
