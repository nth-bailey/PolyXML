#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# PolyXML Unified Benchmarking Suite
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "======================================================================"
echo "⚡ PolyXML Performance & Benchmarking Suite"
echo "======================================================================"

# 1. Ensure Rust toolchain
if command -v cargo >/dev/null 2>&1; then
    CARGO_BIN="cargo"
elif [ -f "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
    CARGO_BIN="cargo"
else
    echo "❌ Error: cargo not found."
    exit 1
fi

# 2. Ensure Python environment
PYTHON_BIN="${ROOT_DIR}/.venv/bin/python"
if [ ! -f "${PYTHON_BIN}" ]; then
    echo "Creating virtual environment at .venv..."
    uv venv "${ROOT_DIR}/.venv" --python 3.12
fi

echo "Installing benchmark requirements..."
uv pip install -q -r "${SCRIPT_DIR}/requirements.txt" maturin

# 3. Build Rust Python extension in Release mode
echo ""
echo "📦 Building polyxml-python in release mode..."
cd "${ROOT_DIR}/crates/polyxml-python"
"${ROOT_DIR}/.venv/bin/maturin" develop --release

# 4. Run Rust Criterion Benchmarks (Pure Core Engine)
echo ""
echo "🦀 Running Rust polyxml-core Criterion benchmarks..."
cd "${ROOT_DIR}"
"${CARGO_BIN}" bench --bench core_benchmarks

# 5. Run Python Comparative Multi-Parser Benchmarks
echo ""
echo "🐍 Running Python comparative benchmarks..."
cd "${ROOT_DIR}"
"${PYTHON_BIN}" -m benchmarks \
    --workload all \
    --catalog-sizes 1000 10000 \
    --iterations 25 \
    --output-md "${SCRIPT_DIR}/results.md" \
    --output-json "${SCRIPT_DIR}/results.json"

echo ""
echo "======================================================================"
echo "✅ Benchmarking Complete!"
echo "   - Rust reports: target/criterion/"
echo "   - Python Markdown: benchmarks/results.md"
echo "   - Python JSON: benchmarks/results.json"
echo "======================================================================"
