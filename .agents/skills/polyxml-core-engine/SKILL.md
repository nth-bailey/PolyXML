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
