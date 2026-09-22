#!/bin/sh
# Compile-and-run wrapper for scripts/perf_stat.c (see that file for usage).
# Usage: scripts/perf_stat.sh -- <command> [args...]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
# Run the countered command under a RAM cap (scripts/memcap.sh) so runaway
# workloads can't freeze small hosts; the LEVEL guard makes the re-exec
# idempotent. POLYXML_MEMCAP_DISABLE=1 opts out. Note: pass --bench to
# Criterion binaries, otherwise they run test mode (~0.2s) and the counters
# only measure process startup.
if [ -z "${POLYXML_MEMCAP_LEVEL:-}" ]; then
    exec "$here/memcap.sh" "$here/$(basename "$0")" "$@"
fi
bin="${TMPDIR:-/tmp}/polyxml_perf_stat.$$"
trap 'rm -f "$bin"' EXIT
gcc -O2 -o "$bin" "$here/perf_stat.c"
exec "$bin" "$@"
