---
name: polyxml-abi3-workflow
description: >-
  Use this skill when developing, compiling, testing, or debugging the PolyXML Python extension,
  working with PyO3, compiling with maturin, validating Python ABI3 compatibility, or running dual-language verification.
---

# PolyXML Python ABI3 Development & Verification Playbook

This skill describes the complete development, compilation, and testing workflow for the
`polyxml-python` PyO3 extension.

## 1. ABI3 Architecture & Invariants

- **ABI Level**: `abi3-py312` exclusively (Stable Python 3.12+ ABI).
- **Core Isolation**: `crates/polyxml-core` contains **zero** PyO3 or Python runtime dependencies. All Python-specific mapping, conversions, and PyO3 bindings reside in `crates/polyxml-python`.
- **Zero Allocations**: Use `lexical-core` for parsing byte slices directly into native types; avoid copying string slices into intermediate Python strings until necessary.

## 2. Compilation with Maturin

To build and install the native extension in development mode into `.venv`:

```bash
# Activate virtual environment or ensure maturin is in PATH
PATH="$PWD/.venv/bin:$PATH"

# Build and install extension in-place for development
cd crates/polyxml-python
maturin develop

# For release mode performance testing
maturin develop --release
```

## 3. ABI3 Conformance Auditing

Ensure no unstable C-API calls or non-limited API symbols leaked into the compiled shared library:

```bash
# Check shared library with abi3audit
uv run abi3audit $(find ../../target/ -name "polyxml*.so" | head -n 1)
```

## 4. Full Quality & Verification Checklist

Before committing any Python or PyO3 changes:

```bash
# 1. Rust checks across all crates
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 2. Recompile Python extension
cd crates/polyxml-python && maturin develop && cd ../..

# 3. Python linting & formatting
ruff check crates/polyxml-python/python/ tests/
ruff format --check crates/polyxml-python/python/ tests/

# 4. Python test suite with 100% statement & branch coverage
pytest --cov=polyxml --cov-branch --cov-fail-under=100
```
