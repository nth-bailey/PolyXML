#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
wasm-pack build "$repo_dir/crates/polyxml-wasm" --target web --out-dir pkg --release
node "$repo_dir/crates/polyxml-wasm/package.mjs"
