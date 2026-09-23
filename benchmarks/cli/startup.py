#!/usr/bin/env python3
"""Measure fresh CLI processes with warm or evicted executable file pages.

Run on Linux after building the release CLI. Eviction uses POSIX_FADV_DONTNEED
for the CLI executable only; shared libraries and the filesystem may stay warm.
"""

import argparse
import json
import os
import platform
import resource
import statistics
import tempfile
import time
from pathlib import Path


def run_once(
    binary: Path, args: tuple[str, ...], expected_status: int
) -> dict[str, float | int]:
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.perf_counter_ns()
    pid = os.posix_spawn(
        binary,
        (str(binary), *args),
        os.environ,
        file_actions=(
            (os.POSIX_SPAWN_OPEN, 1, os.devnull, os.O_WRONLY, 0),
            (os.POSIX_SPAWN_OPEN, 2, os.devnull, os.O_WRONLY, 0),
        ),
    )
    _, status = os.waitpid(pid, 0)
    elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
    actual_status = os.waitstatus_to_exitcode(status)
    if actual_status != expected_status:
        raise RuntimeError(
            f"{' '.join(args)} exited {actual_status}, expected {expected_status}"
        )
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    return {
        "elapsed_ms": elapsed_ms,
        "major_faults": after.ru_majflt - before.ru_majflt,
    }


def evict_executable(binary: Path) -> None:
    with binary.open("rb") as executable:
        os.posix_fadvise(executable.fileno(), 0, 0, os.POSIX_FADV_DONTNEED)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/polyxml"))
    parser.add_argument("--runs", type=int, default=30)
    parser.add_argument(
        "--output", type=Path, default=Path("benchmarks/cli/target/startup.json")
    )
    args = parser.parse_args()
    if args.runs < 1:
        parser.error("--runs must be positive")
    if not hasattr(os, "posix_fadvise"):
        parser.error("POSIX_FADV_DONTNEED requires Linux/POSIX")
    binary = args.binary.resolve(strict=True)

    with tempfile.TemporaryDirectory() as temp_dir:
        missing_schema = str(Path(temp_dir) / "missing.xsd")
        cases = {
            "help": (("--help",), 0),
            "argument-validation": (
                ("generate", missing_schema, "--lang", "ts", "--backend", "zodd"),
                2,
            ),
        }
        results = {}
        for name, (command, expected_status) in cases.items():
            for _ in range(5):
                run_once(binary, command, expected_status)
            samples = {"warm": [], "executable-evicted": []}
            for _ in range(args.runs):
                samples["warm"].append(run_once(binary, command, expected_status))
                evict_executable(binary)
                samples["executable-evicted"].append(
                    run_once(binary, command, expected_status)
                )
            results[name] = {}
            for mode, observations in samples.items():
                times = [observation["elapsed_ms"] for observation in observations]
                summary = {
                    "mean_ms": statistics.mean(times),
                    "median_ms": statistics.median(times),
                    "p95_ms": sorted(times)[max(0, (95 * len(times) + 99) // 100 - 1)],
                    "min_ms": min(times),
                    "max_ms": max(times),
                    "runs_with_major_faults": sum(
                        observation["major_faults"] > 0 for observation in observations
                    ),
                    "samples": observations,
                }
                results[name][mode] = summary
                print(
                    f"{name} {mode}: mean {summary['mean_ms']:.2f} ms, "
                    f"median {summary['median_ms']:.2f} ms, "
                    f"major faults in {summary['runs_with_major_faults']}/{args.runs} runs"
                )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(
            {
                "method": "fresh process via posix_spawn; executable file pages requested evicted via POSIX_FADV_DONTNEED; shared libraries and schema cache untouched",
                "binary": str(binary),
                "binary_size_bytes": binary.stat().st_size,
                "host": platform.platform(),
                "runs_per_case_and_mode": args.runs,
                "results": results,
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
