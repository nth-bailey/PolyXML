## Description

Provide a clear and concise summary of what this pull request changes and why.

---

## Type of Change

- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to break)
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Language binding update (Rust / Python / C++ / Go / Java / TypeScript)

---

## Verification Checklist

Before submitting, please ensure:

- [ ] `cargo fmt --all --check` passes with zero differences.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` produces 0 warnings.
- [ ] `cargo test --workspace` passes cleanly.
- [ ] Python tests pass with 100% statement and branch coverage (`pytest --cov=polyxml --cov-fail-under=100`).
- [ ] Binding tests pass for any modified language (`go test`, `ctest`, `npm test`, `mvn test`).
- [ ] New or updated documentation is included if applicable.
