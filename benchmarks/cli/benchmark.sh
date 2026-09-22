#!/usr/bin/env bash
# Measure fresh-process CLI startup and argument validation with hyperfine.
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$repo_root"
command -v hyperfine >/dev/null || { echo 'Install hyperfine to run this benchmark.' >&2; exit 1; }
cargo build --release -p polyxml-cli
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT
export POLYXML_CLI_BENCH_SCHEMA="$scratch_dir/minimal.xsd"
printf '%s\n' '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"/>' > "$POLYXML_CLI_BENCH_SCHEMA"
mkdir -p benchmarks/cli/target
hyperfine --shell=none --warmup 5 --runs 100 \
    --export-json benchmarks/cli/target/cli.json \
    --command-name help 'target/release/polyxml --help' \
    --command-name generate-help 'target/release/polyxml generate --help' \
    --command-name parse-and-validate "target/release/polyxml generate \"$POLYXML_CLI_BENCH_SCHEMA\" --lang java --backend jackson --style pojo --feature builder --feature direct-codec --dry-run"
