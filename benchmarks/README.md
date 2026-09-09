# PolyXML Benchmarking Suite

A comprehensive, reusable benchmarking framework for evaluating the performance, throughput, latency, and memory footprint of **PolyXML** across both its pure Rust core engine (`polyxml-core`) and its Python runtime bindings (`polyxml-python`).

---

## Architecture & Scope

```
benchmarks/
├── README.md               # Architecture, methodology, and usage guide
├── requirements.txt        # Optional comparison dependencies (lxml, xsdata, etc.)
├── run_all.sh              # One-command runner for Rust and Python benchmarks
├── generators.py           # Deterministic XML payload generators
├── models.py               # Target schemas for PolyXML, xsdata, pydantic-xml
├── runner.py               # Benchmark execution engine (timing, tracemalloc, stats)
├── __main__.py             # CLI runner interface (rich tables, JSON/MD export)
├── results.md              # Exported Markdown table (generated on run)
└── results.json            # Exported machine-readable JSON metrics (generated on run)
```

### 1. Pure Rust Core Benchmarks (`polyxml-core`)
Located in [`crates/polyxml-core/benches/core_benchmarks.rs`](../crates/polyxml-core/benches/core_benchmarks.rs), this suite uses [Criterion.rs](https://github.com/bheisler/criterion.rs) to perform statistically rigorous latency and throughput benchmarks of:
- **Sensor Micro Payload (~130B)**: Real-time telemetry command parsing.
- **Catalog Batch Streaming (1,000 & 10,000 items)**: Measures gigabytes/sec and megabytes/sec streaming throughput for both deserialization and serialization.

### 2. Python Comparative Benchmarking Suite (`benchmarks/`)
Compares **PolyXML** (`polyxml.deserialize` and `polyxml.serialize`) against industry-standard Python XML parsers and data-binding libraries:
- **`xsdata`**: Standard Python XML data binder creating dataclasses.
- **`pydantic-xml`**: XML data binder producing Pydantic v2 models.
- **`lxml.etree`**: High-performance C (`libxml2`) + Cython DOM parser.
- **`xml.etree.ElementTree`**: Standard library Python DOM parser.

---

## Quickstart

### Prerequisites

- Rust 1.80+ (`cargo`, `rustc`)
- Python 3.12+ (or managed via `uv`)

### Run Everything with One Command

```bash
./benchmarks/run_all.sh
```

This will:
1. Compile `polyxml-python` in `--release` mode using `maturin`.
2. Run Criterion benchmarks for `polyxml-core`.
3. Run the Python multi-parser suite across all workloads and export `benchmarks/results.md` and `benchmarks/results.json`.

---

## Running Individual Benchmarks

### Rust Criterion Benchmarks

To run the pure Rust core benchmarks:

```bash
cargo bench --bench core_benchmarks
```

To run a specific benchmark (e.g. only deserialization):
```bash
cargo bench --bench core_benchmarks -- deserialization
```

Criterion reports and HTML charts are generated in `target/criterion/report/index.html`.

### Python Comparative Benchmarks

Activate your virtual environment and run the CLI:

```bash
python -m benchmarks --workload all --catalog-sizes 1000 10000 --iterations 25 --output-md benchmarks/results.md --output-json benchmarks/results.json
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
1. Define a generator function in `benchmarks/generators.py`.
2. Define corresponding dataclasses and models in `benchmarks/models.py`.
3. Add a `run_workload_<name>` method to `BenchmarkRunner` in `benchmarks/runner.py`.
4. Expose the workload choice in `benchmarks/__main__.py`.
