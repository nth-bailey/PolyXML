#!/bin/sh
# memcap.sh - run a command under a hard RAM cap.
#
# Benchmarks and codegen smoke tests build large generated crates; on small
# dev machines (e.g. a 7.7-GiB WSL box) an uncapped `cargo build --release`
# can exhaust RAM, drive the host into swap-thrash and freeze it. This
# wrapper caps the command's whole process tree so an over-limit build is
# OOM-killed *inside its own cgroup* (exit 137) while the host stays up.
#
# Usage: scripts/memcap.sh <command> [args...]
#
# Environment:
#   POLYXML_MEMCAP_PCT      Percentage of currently-available RAM the command
#                           may use (default: 60). Integer 1-100.
#   POLYXML_MEMCAP_BACKEND  auto (default) | systemd | ulimit | none
#                             auto   - Linux with a systemd --user manager:
#                                      kernel-enforced MemoryMax (preferred);
#                                      otherwise fall back to ulimit.
#                             systemd - require a systemd --user scope or fail
#                                       (useful for asserting the hard cap).
#                             ulimit  - force the address-space fallback
#                                       (no systemd, containers, macOS).
#                             none    - run uncapped.
#   POLYXML_MEMCAP_DISABLE  Set to 1 to run uncapped (same as backend=none).
#   POLYXML_MEMCAP_LEVEL    Nesting counter; set automatically so wrappers
#                           (e.g. perf_stat.sh) can re-enter safely without
#                           double-capping.
#
# Notes:
#   * The percentage is recomputed against *available* RAM at each invocation.
#   * The systemd path sets MemorySwapMax=0 so capped work can never page the
#     host into thrash: hitting the cap fails fast (kernel OOM kill) instead
#     of grinding to a halt.
#   * The ulimit path caps address space (RLIMIT_AS), which is best-effort:
#     it reliably stops runaway allocations but counts reserved -not- resident
#     pages, so it may be stricter than the systemd cap.
set -eu

usage() {
    echo "usage: $0 <command> [args...]" >&2
    exit 2
}

[ "$#" -gt 0 ] || usage

pct="${POLYXML_MEMCAP_PCT:-60}"
backend="${POLYXML_MEMCAP_BACKEND:-auto}"

case "$pct" in
    '' | *[!0-9]*)
        echo "memcap: POLYXML_MEMCAP_PCT must be an integer (got '$pct')" >&2
        exit 2
        ;;
esac
if [ "$pct" -lt 1 ] || [ "$pct" -gt 100 ]; then
    echo "memcap: POLYXML_MEMCAP_PCT must be 1-100 (got '$pct')" >&2
    exit 2
fi

# Mark ourselves as "inside a cap" BEFORE any exec-out below: wrapped
# scripts re-exec through memcap when POLYXML_MEMCAP_LEVEL is unset, so the
# disable/fallback paths must still set it or they would loop forever.
level=$((${POLYXML_MEMCAP_LEVEL:-0} + 1))
export POLYXML_MEMCAP_LEVEL="$level"

if [ "${POLYXML_MEMCAP_DISABLE:-0}" = "1" ] || [ "$backend" = "none" ]; then
    exec "$@"
fi

# --- How much RAM may we take? -------------------------------------------
os="$(uname -s)"
avail_kb=""

if [ "$os" = "Linux" ] && [ -r /proc/meminfo ]; then
    avail_kb="$(awk '/^MemAvailable:/ {print $2; exit}' /proc/meminfo)"
fi
if [ -z "$avail_kb" ] && [ "$os" = "Darwin" ]; then
    # Best effort: free + inactive + speculative pages (macOS reclaims the
    # latter two on demand, so they are effectively available).
    page_size="$(sysctl -n hw.pagesize 2>/dev/null || echo 4096)"
    avail_kb="$(vm_stat 2>/dev/null | awk -v ps="$page_size" '
        /Pages free:/      {gsub(/\./, "", $3); free = $3}
        /Pages inactive:/  {gsub(/\./, "", $3); inact = $3}
        /Pages speculative:/{gsub(/\./, "", $3); spec = $3}
        END {if (free == "") exit 1; printf "%d", (free + inact + spec) * ps / 1024}')" || avail_kb=""
fi
if [ -z "$avail_kb" ] || [ "$avail_kb" -le 0 ] 2>/dev/null; then
    # Last resort: total RAM.
    if [ "$os" = "Darwin" ]; then
        total_bytes="$(sysctl -n hw.memsize 2>/dev/null || echo 0)"
        avail_kb=$((total_bytes / 1024))
    elif [ -r /proc/meminfo ]; then
        avail_kb="$(awk '/^MemTotal:/ {print $2; exit}' /proc/meminfo)"
    fi
fi
if [ -z "$avail_kb" ] || [ "$avail_kb" -le 0 ] 2>/dev/null; then
    echo "memcap: cannot determine available memory; running uncapped" >&2
    exec "$@"
fi

cap_kb=$((avail_kb * pct / 100))
cap_mib=$((cap_kb / 1024))
avail_mib=$((avail_kb / 1024))

# --- Backend: kernel-enforced cgroup (preferred) --------------------------
use_systemd=0
if [ "$backend" = "systemd" ] ||
    { [ "$backend" = "auto" ] && [ "$os" = "Linux" ] && command -v systemd-run >/dev/null 2>&1; }; then
    # Probe the *user* manager once. Works in systemd distros and CI images
    # with a session bus; fails in bare containers/chroots -> fall back.
    if systemd-run --user --scope --quiet -p MemoryMax=64M -- /bin/true \
        </dev/null >/dev/null 2>&1; then
        use_systemd=1
    elif [ "$backend" = "systemd" ]; then
        echo "memcap: systemd --user scope unavailable but POLYXML_MEMCAP_BACKEND=systemd was requested" >&2
        exit 2
    fi
fi

if [ "$use_systemd" = 1 ]; then
    echo "memcap: <=${cap_mib} MiB (${pct}% of ${avail_mib} MiB available) [systemd MemoryMax, swap off]" >&2
    exec systemd-run --user --scope --quiet \
        --setenv=POLYXML_MEMCAP_LEVEL="$level" \
        -p "MemoryMax=${cap_kb}K" \
        -p MemorySwapMax=0 \
        -- "$@"
fi

# --- Fallback: RLIMIT_AS --------------------------------------------------
if [ "$backend" = "auto" ] || [ "$backend" = "ulimit" ]; then
    echo "memcap: <=${cap_mib} MiB (${pct}% of ${avail_mib} MiB available) [ulimit -v fallback]" >&2
    if ulimit -v "$cap_kb" 2>/dev/null; then
        exec "$@"
    fi
    echo "memcap: could not lower RLIMIT_AS (hard limit too low?); running uncapped" >&2
    exec "$@"
fi

echo "memcap: unknown POLYXML_MEMCAP_BACKEND '$backend'" >&2
exit 2
