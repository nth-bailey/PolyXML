# PolyXML Benchmark Suites

Benchmarking tooling for PolyXML's Rust core and its language bindings. The
directory is organized as **one self-contained directory per language** plus a
cross-language orchestrator, so new suites never crowd the root.

## Layout

```
benchmarks/
├── README.md        # This index: catalog, methodology, conventions
├── run_all.sh       # Orchestrator: builds the extension, runs Rust + Python suites
├── __init__.py      # Package marker for `python -m benchmarks.python`
├── cli/             # CLI startup and argument validation (hyperfine)
├── python/          # Python bindings vs the Python XML ecosystem (custom CLI suite)
└── java/            # Java bindings vs JAXB, Jackson, and the Panama native binding (JMH)
```

Each suite directory owns its `README.md`, runner, dependencies, and generated
results; generated output stays in the suite's gitignored `target/` (or
equivalent) and never at this root.

## Catalog

| Suite | Location | Tooling | How to run |
| :--- | :--- | :--- | :--- |
| Rust core engine | [`crates/polyxml-core/benches/`](../crates/polyxml-core/benches/) | [Criterion.rs](https://github.com/bheisler/criterion.rs) | `cargo bench --bench core_benchmarks` |
| Python comparative | [`python/`](python/README.md) | Custom CLI suite | `python -m benchmarks.python` or `./benchmarks/run_all.sh` |
| CLI startup | [`cli/`](cli/README.md) | hyperfine | `./benchmarks/cli/benchmark.sh` |
| Java four-runtime | [`java/`](java/README.md) | JMH (Maven) | See [`java/README.md`](java/README.md) |

The Rust Criterion suite lives inside its crate because `cargo bench` requires
`benches/` next to the crate manifest — it is the one deliberate exception to the
one-directory-per-suite rule. Filter a benchmark and open the HTML report:

```bash
cargo bench --bench core_benchmarks -- deserialization
# Report: target/criterion/report/index.html
```

### Run everything (Rust + Python)

```bash
./benchmarks/run_all.sh
```

Requires Rust 1.80+ and Python 3.12+ (or `uv`). The script compiles
`polyxml-python` in `--release`, runs Criterion, then runs the Python suite and
writes `benchmarks/python/results.md` / `results.json`.

---

## Shared methodology rules

Suite READMEs document their own tooling; these rules apply to all of them:

1. **Smoke runs prove execution, not speed.** Short iterations and single forks
   are execution checks only — never quote them as performance evidence.
2. **Measure on an otherwise idle host** and record the toolchain, OS/CPU, and
   repository revision next to the numbers.
3. **Inspect confidence intervals and repeat** before making any throughput
   claim; a single run on a shared machine is noise.
4. Published figures live in [`docs/benchmarks.md`](../docs/benchmarks.md) —
   update them from a full, idle-host run, not from a smoke.

## Adding a suite

1. Create `benchmarks/<language>/` and copy [`java/README.md`](java/README.md) as
   a template: it covers build, run commands, workloads, interpretation caveats,
   and profiler usage.
2. Keep the suite self-contained — build files, generators, tests, and results all
   live under the suite directory, with generated output gitignored.
3. Do not add loose files to this root; it holds only this index, the
   orchestrator, and the package marker.
4. If `run_all.sh` should drive the suite, gate it on toolchain detection so
   machines without that toolchain still succeed.
5. Add the suite to the catalog above and link it from
   [`docs/benchmarks.md`](../docs/benchmarks.md).

---

Published methodology, result tables, and reproduction commands:
[`docs/benchmarks.md`](../docs/benchmarks.md).
