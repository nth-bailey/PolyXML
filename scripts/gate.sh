#!/usr/bin/env bash
# ==============================================================================
# scripts/gate.sh - Unified Dual-Language Quality Gate for PolyXML
# ==============================================================================
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# Ensure virtual environment binaries take precedence in PATH
export PATH="${REPO_ROOT}/.venv/bin:${PATH}"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}==> [1/6] Running Rust format check (cargo fmt)...${NC}"
cargo fmt --check

echo -e "${BLUE}==> [2/6] Running Rust Clippy (cargo clippy -D warnings)...${NC}"
cargo clippy --workspace --all-targets -- -D warnings

echo -e "${BLUE}==> [3/6] Running Rust workspace tests (cargo test)...${NC}"
cargo test --workspace

echo -e "${BLUE}==> [4/6] Compiling Python PyO3 extension (maturin develop)...${NC}"
(
  cd crates/polyxml-python
  export PATH="${PWD}/.venv/bin:${PATH}"
  maturin develop
  
  echo -e "${BLUE}==> [5/6] Running Python Ruff checks...${NC}"
  ruff check python/ tests/
  ruff format --check python/ tests/

  echo -e "${BLUE}==> [6/6] Running Python tests with 100% statement & branch coverage...${NC}"
  pytest --cov=polyxml --cov-branch --cov-fail-under=100 -q
)

echo -e "${GREEN}✨ All PolyXML dual-language quality gates passed!${NC}"
