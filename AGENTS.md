# AGENTS.md

Instructions and guidelines for AI coding assistants working in the `PolyXML`
repository.

---

## 1. Project Overview

`PolyXML` is the high-performance, polyglot native XML data-binding engine, providing
ultra-fast bidirectional XML serialization and deserialization across programming
languages, starting with Rust and Python (`dataclasses` and Pydantic v2).

- **Technology**: Rust 2021, PyO3 (`abi3-py312`), `quick-xml`, `lexical-core`, `smallvec`.
- **Repository**: `nth-bailey/PolyXML`
- **Supported Python**: `Python >= 3.12` exclusively.
- **Maintainer**: Bailey Nguyen (`bailey.tan.nguyen@gmail.com`).

---

## 2. Core Architectural Decisions & Invariants

When contributing or refactoring, strictly maintain the following invariants:

1. **Pure Rust Core Engine (`crates/polyxml-core`)**:
   - The core engine MUST have **zero Python, PyO3, or runtime-specific dependencies**.
   - All parsing, serialization, and schema IR logic must remain 100% pure, idiomatic Rust.

2. **Python ABI3 Portability (`abi3-py312`)**:
   - The Python extension (`crates/polyxml-python`) is compiled against the stable Python 3.12+ ABI (`abi3`).
   - Do not introduce non-limited C-API calls that violate `abi3-py312` compatibility.

3. **Zero Unnecessary Allocations**:
   - Stream XML tokens using `quick-xml` reader/writer events.
   - Use `lexical-core` for high-throughput integer and float conversions from byte slices.
   - Avoid intermediate DOM allocations during parsing.

4. **Bidirectional Serialization & Deserialization**:
   - Features must support both reading (XML -> Object) and writing (Object -> XML).

5. **100% Code Coverage**:
   - All Python wrapper code in `crates/polyxml-python/python/` must maintain **100%
     statement and branch test coverage** (`fail_under = 100` in `pyproject.toml`).
   - All new features or bug fixes must include corresponding tests in `tests/`.

---

## 3. Tooling & Development Workflow

### Rust Toolchain & Build

- Build workspace:
  ```bash
  cargo check --workspace
  ```
- Run Rust tests:
  ```bash
  cargo test --workspace
  ```

### Rust Linting & Formatting

- Formatting:
  ```bash
  cargo fmt --check
  ```
- Clippy:
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

### Python Linting & 100% Coverage Testing

- Build Python extension:
  ```bash
  cd crates/polyxml-python
  maturin develop
  ```
- Run Python tests with coverage:
  ```bash
  pytest --cov=polyxml --cov-branch --cov-fail-under=100
  ```
- Python Linting:
  ```bash
  ruff check python/ tests/
  ```

---

## 4. Verification Checklist

Before completing any task:

1. **Rust Format**: Ensure `cargo fmt --check` passes with zero differences.
2. **Rust Clippy**: Ensure `cargo clippy --workspace --all-targets -- -D warnings` produces 0 warnings.
3. **Rust Tests**: Ensure `cargo test --workspace` passes cleanly.
4. **Python Tests & Coverage**: Ensure `pytest --cov=polyxml --cov-branch --cov-fail-under=100` passes with **100% coverage**.
5. **Python Lint**: Ensure `ruff check python/ tests/` passes with 0 errors.
