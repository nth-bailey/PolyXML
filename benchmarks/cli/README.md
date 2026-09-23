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

## Controlled executable-cache measurement

On Linux, build the release CLI and run:

```bash
./scripts/memcap.sh cargo build --release -p polyxml-cli
python3 benchmarks/cli/startup.py --runs 100
```

`startup.py` times each fresh process from `posix_spawn` through exit. It
alternates warm runs with runs after `POSIX_FADV_DONTNEED` requests eviction of
the CLI executable's file pages. It measures top-level help and an invalid
backend that fails during argument validation, before schema I/O. It records
individual times and major page faults in `benchmarks/cli/target/startup.json`.
This is a **cold executable-cache proxy**, not a first run after reboot: shared
libraries, the loader, Python benchmark process, and filesystem caches outside
the CLI executable may remain warm. Verify that evicted runs have major faults;
otherwise the eviction request did not establish the intended condition.

On a Linux 6.18 WSL2 x86-64 host with an ext4 filesystem and a 2,296,776-byte
release CLI, 100 runs per case and mode on 2026-09-22 measured:

| Case | Cache state | Mean | Median | p95 | Runs with major faults |
| --- | --- | ---: | ---: | ---: | ---: |
| `--help` | Warm | 1.57 ms | 1.52 ms | 1.94 ms | 0/100 |
| `--help` | Executable evicted | 7.64 ms | 7.61 ms | 8.15 ms | 100/100 |
| Invalid backend | Warm | 1.66 ms | 1.64 ms | 1.99 ms | 0/100 |
| Invalid backend | Executable evicted | 7.75 ms | 7.71 ms | 8.43 ms | 100/100 |

The proposed <5 ms target holds for warm caches but **does not hold** for this
host's executable-evicted runs. The major faults and roughly 6 ms gap point to
file loading as the main cost on this host; the measurement does not establish
which part is inherent or what another storage system would do. A universal
post-reboot target requires measurements on representative machines. Issue #63
tracks the remaining cold-start target investigation.

## Error display

Misspelled and incompatible backends and invalid features produce an actionable
message listing supported values. On a terminal with `TERM=xterm-256color` and
no `NO_COLOR`, clap colors the error label and usage text. Redirected output and
terminals with `NO_COLOR=1` contain no ANSI escape codes. These three cases
were checked with both a pseudo-terminal and captured stderr on the same host;
the CLI needed no diagnostic code change.
