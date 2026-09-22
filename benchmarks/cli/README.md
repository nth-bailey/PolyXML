# CLI startup and argument validation

Run from the repository root with Rust and `hyperfine` installed:

```bash
./benchmarks/cli/benchmark.sh
```

The runner builds the release CLI, creates an empty temporary XSD, and measures
100 fresh processes per command after five warmups. `--shell=none` excludes
shell startup and avoids shell-calibration noise for sub-5ms commands. JSON
results are saved to `benchmarks/cli/target/cli.json` (gitignored).

| Case | Work measured |
| --- | --- |
| `help` | Process startup and top-level help rendering |
| `generate-help` | Process startup and generation help rendering |
| `parse-and-validate` | Argument parsing, Java backend/style/features validation, and parsing an empty XSD in dry-run mode |

A local Linux x86-64 release build measured on 2026-09-22 with hyperfine 1.19.0:

| Case | Mean ± standard deviation |
| --- | --- |
| `help` | 1.4 ± 0.2 ms |
| `generate-help` | 1.5 ± 0.2 ms |
| `parse-and-validate` | 1.6 ± 0.2 ms |

These measurements meet issue #50's 5ms target on this host. They use warm OS
caches: they do **not** establish cold-disk or post-reboot startup latency.
The parsing case includes tiny-schema I/O and parsing, so it is not an isolated
argument-parser microbenchmark. Results depend on hardware and system load;
rerun the script to compare changes on the same host.
