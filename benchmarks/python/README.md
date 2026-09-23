# Python comparative benchmark suite

Compares **PolyXML** (`polyxml.deserialize` and `polyxml.serialize`) against
industry-standard Python XML parsers and data-binding libraries across latency,
throughput, and memory footprint:

- **`xsdata`**: Standard Python XML data binder creating dataclasses.
- **`pydantic-xml`**: XML data binder producing Pydantic v2 models.
- **`lxml.etree`**: High-performance C (`libxml2`) + Cython DOM parser.
- **`xml.etree.ElementTree`**: Standard library Python DOM parser.
- Further comparisons (`xmltodict`, `defusedxml`, `declxml`, `untangle`, ...) are
  enabled automatically when installed — see [`requirements.txt`](requirements.txt).

---

## Layout

```
python/
├── README.md         # This guide
├── requirements.txt  # Optional comparison dependencies (lxml, xsdata, etc.)
├── generators.py     # Deterministic XML payload generators
├── models.py         # Target schemas for PolyXML, xsdata, pydantic-xml
├── runner.py         # Benchmark execution engine (timing, tracemalloc, stats)
├── __main__.py       # CLI runner interface (rich tables, JSON/MD export)
├── results.md        # Exported Markdown table (generated on run)
└── results.json      # Exported machine-readable JSON metrics (generated on run)
```

---

## Quickstart

Prerequisites: Rust 1.80+ (`cargo`, `rustc`) and Python 3.12+ (or managed via
`uv`), with the `polyxml` extension built in release mode (`maturin develop
--release` from `crates/polyxml-python`).

Run the whole Rust + Python pipeline from the repository root with one command:

```bash
./benchmarks/run_all.sh
```

This builds the extension, runs the Rust Criterion suite, then this suite —
exporting `benchmarks/python/results.md` and `benchmarks/python/results.json`.

### Run this suite directly

Activate your virtual environment, then from the repository root:

```bash
python -m benchmarks.python --workload all --catalog-sizes 1000 10000 --iterations 25 --output-md benchmarks/python/results.md --output-json benchmarks/python/results.json
```

#### CLI Options

| Flag | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `--workload` | `choice` | `all` | Select workload: `all`, `sensor`, `order`, or `catalog`. |
| `--catalog-sizes` | `int ...` | `1000 10000` | Number of items in streaming catalog payloads. |
| `--iterations` | `int` | `25` | Number of timed iterations per test. |
| `--output-md` | `path` | `None` | Path to export a GitHub Flavored Markdown summary table. |
| `--output-json` | `path` | `None` | Path to export machine-readable JSON metrics. |

---

## Metrics Measured

1. **Min Latency**: The minimum execution time across iterations (removes OS scheduling jitter).
2. **Median & P95 Latency**: Distribution percentiles representing real-world throughput stability.
3. **Throughput (MB/s)**: $\frac{\text{Payload Size (MB)}}{\text{Execution Time (s)}}$.
4. **Peak Memory (KiB)**: Monitored via Python's `tracemalloc` to assess heap allocation overhead.
5. **Speedup vs Baseline**: Relative acceleration factor calculated against pure Python typed data binding (`xsdata`).

---

## Adding Custom Workloads

To add a new benchmark scenario:
1. Define a generator function in `benchmarks/python/generators.py`.
2. Define corresponding dataclasses and models in `benchmarks/python/models.py`.
3. Add a `run_workload_<name>` method to `BenchmarkRunner` in `benchmarks/python/runner.py`.
4. Expose the workload choice in `benchmarks/python/__main__.py`.

---

Published result tables live in [`docs/benchmarks/index.md`](../../docs/benchmarks/index.md);
suite conventions and the full catalog are in the [suite index](../README.md).
