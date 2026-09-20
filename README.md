# PolyXML

<p align="center">
  <strong>The "protoc for XML" — Modern Polyglot Schema Compiler & Ultra-Fast Streaming Runtime</strong>
</p>

<p align="center">
  <a href="https://github.com/nth-bailey/PolyXML/actions"><img src="https://img.shields.io/github/actions/workflow/status/nth-bailey/PolyXML/ci.yml?branch=main&label=CI&logo=github" alt="CI"></a>
  <a href="https://nth-bailey.github.io/PolyXML/"><img src="https://img.shields.io/badge/docs-zensical-blue.svg?logo=gitbook" alt="Docs"></a>
  <a href="https://github.com/nth-bailey/polyxml-w3c-tests"><img src="https://img.shields.io/badge/W3C%20XSTS-99.8%25%20Passed-brightgreen.svg" alt="W3C XSTS Conformance"></a>
  <a href="https://github.com/nth-bailey/PolyXML/blob/main/crates/polyxml-python/pyproject.toml"><img src="https://img.shields.io/badge/coverage-100%25-brightgreen.svg?logo=pytest" alt="Coverage: 100%"></a>
  <a href="https://github.com/astral-sh/ruff"><img src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json" alt="Ruff"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/polyxml"><img src="https://img.shields.io/crates/v/polyxml.svg?logo=rust&label=crates.io" alt="crates.io: polyxml"></a>
  <a href="https://crates.io/crates/polyxml-cli"><img src="https://img.shields.io/crates/v/polyxml-cli.svg?logo=rust&label=polyxml-cli" alt="crates.io: polyxml-cli"></a>
  <a href="https://pypi.org/project/polyxml/"><img src="https://img.shields.io/pypi/v/polyxml.svg?logo=pypi&label=PyPI" alt="PyPI: polyxml"></a>
  <a href="https://www.npmjs.com/package/polyxml"><img src="https://img.shields.io/npm/v/polyxml.svg?logo=npm&color=CB3837&label=npm" alt="npm: polyxml"></a>
  <a href="https://central.sonatype.com/artifact/io.github.nth-bailey/polyxml"><img src="https://img.shields.io/maven-central/v/io.github.nth-bailey/polyxml.svg?logo=apache-maven&color=C71A36&label=Maven" alt="Maven Central"></a>
  <a href="https://pkg.go.dev/github.com/nth-bailey/PolyXML/bindings/go"><img src="https://pkg.go.dev/badge/github.com/nth-bailey/PolyXML/bindings/go.svg" alt="Go Reference"></a>
  <a href="https://github.com/nth-bailey/homebrew-polyxml"><img src="https://img.shields.io/badge/Homebrew-polyxml-FBB040.svg?logo=homebrew&logoColor=black" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust" alt="Rust: 1.80+"></a>
  <a href="https://www.python.org"><img src="https://img.shields.io/badge/Python-3.12%20%7C%203.13%20%7C%203.14%20%7C%203.15-3776AB.svg?logo=python&logoColor=white" alt="Python: 3.12 | 3.13 | 3.14 | 3.15"></a>
  <a href="https://nodejs.org"><img src="https://img.shields.io/badge/Node.js-20%20%7C%2022-339933.svg?logo=node.js&logoColor=white" alt="Node.js: 20 | 22"></a>
  <a href="https://www.typescriptlang.org"><img src="https://img.shields.io/badge/TypeScript-5.0%2B-3178C6.svg?logo=typescript&logoColor=white" alt="TypeScript: 5.0+"></a>
  <a href="https://openjdk.org/projects/panama/"><img src="https://img.shields.io/badge/Java-22%2B%20Panama-ED8B00.svg?logo=openjdk&logoColor=white" alt="Java: 22+ Panama"></a>
  <a href="https://go.dev"><img src="https://img.shields.io/badge/Go-1.22%2B-00ADD8.svg?logo=go&logoColor=white" alt="Go: 1.22+"></a>
  <a href="https://en.cppreference.com/w/cpp/20"><img src="https://img.shields.io/badge/C%2B%2B-20-00599C.svg?logo=c%2B%2B" alt="C++: 20"></a>
  <a href="https://dotnet.microsoft.com"><img src="https://img.shields.io/badge/.NET-8.0%2B-512BD4.svg?logo=dotnet&logoColor=white" alt=".NET: 8.0+"></a>
</p>

---

## Overview

**PolyXML** is the next-generation polyglot XML schema compiler and ultra-fast streaming data-binding runtime built in safe Rust.

Just as Protocol Buffers (`protoc`) and FlatBuffers (`flatc`) modernized binary serialization, **PolyXML brings modern software engineering to XML**:

1. **🛠️ Polyglot Schema Compiler (`polyxml`)**: Ingests W3C XSD 1.0 and 1.1 schemas, resolves cyclic types with Tarjan's SCC algorithm, and compiles production-ready, strongly-typed data contracts across **7 modern ecosystems** simultaneously (**Python**, **Rust**, **C++**, **Java**, **TypeScript**, **Go**, and **C#**).
2. **⚡ Ultra-Fast Streaming Runtime**: Direct-to-struct deserialization and serialization powered by `quick-xml` and `lexical-core`, executing **16x–38x faster than traditional tools** with **zero intermediate DOM allocations**.
3. **🏛️ Official W3C XSTS Conformance Tested**: Validated against the official W3C XML Schema Test Suite with a **>99.8% schema compilation pass rate** and **>96% round-trip validation rate** via [polyxml-w3c-tests](https://github.com/nth-bailey/polyxml-w3c-tests).
4. **📦 Permissive MIT License**: 100% open source with zero commercial licensing fees, eliminating the GPL dual-licensing traps of legacy C++ tools.

---

## ⚡ Schema Compilation at a Glance

Compile any W3C XML Schema into strongly-typed code for all 7 languages in a single command:

```bash
# 1. Multi-target compilation in a single invocation
polyxml generate \
  --lang python --backend pydantic-v2 \
  --lang rust --zero-copy --codecs \
  --lang csharp --namespace Enterprise.Banking \
  --lang java --package com.enterprise.banking \
  --lang typescript --zod \
  --lang go --package banking \
  --lang cpp \
  --out ./generated \
  schemas/pain.001.001.09.xsd

# 2. Or build an entire enterprise project declaratively
polyxml build --config polyxml.toml
```

### Consume the Generated Models Instantly

=== "Python 3.12+"
```python
# Generated by polyxml generate --lang python
from generated.python import Customer
import polyxml

# 16x faster XML parsing with zero intermediate DOM overhead
customer = Customer.from_xml(xml_bytes)
xml_output = customer.to_xml(indent=2)

# 10x faster native JSON — completely replace xsdata:
json_bytes = customer.to_json(indent=2)
customer = Customer.from_json(json_bytes)
```

=== "Rust (Zero-Copy)"
```rust
// Generated by polyxml generate --lang rust --zero-copy --codecs
use generated::rust::Customer;

// Zero-copy deserialization: borrows text slices directly with Cow<'a, str>
let customer = Customer::from_xml(xml_str)?;
assert_eq!(customer.name.as_ref(), "Alice");

// Stream back to XML or native JSON
let output_xml = customer.to_xml_string()?;
let output_json = customer.to_json_string()?;
```

=== "Go 1.22+"
```go
// Generated by polyxml generate --lang go
// Models are dual-annotated with xml:"..." and json:"..." struct tags
package main

import (
    "encoding/json"
    "encoding/xml"
    "fmt"
    "generated/banking"
)

func main() {
    var customer banking.Customer
    _ = xml.Unmarshal(xmlBytes, &customer)
    
    // Marshal to JSON immediately — zero translation layer:
    jsonBytes, _ := json.MarshalIndent(customer, "", "  ")
    fmt.Println(string(jsonBytes))
}
```

=== "C# 12 / .NET 8+"
```csharp
// Generated by polyxml generate --lang csharp
// Records feature both XmlSerializer and System.Text.Json attributes
using Enterprise.Banking;
using System.Text.Json;
using System.Xml.Serialization;

var serializer = new XmlSerializer(typeof(Customer));
var customer = (Customer)serializer.Deserialize(new StringReader(xml))!;

// Natively serialize to JSON with System.Text.Json:
string jsonString = JsonSerializer.Serialize(customer);
Console.WriteLine($"Customer {customer.Name} serialized to JSON.");
```

---

## 🔄 Dual-Format XML ↔ JSON Streaming Transcoder (`polyxml transcode`)

Modern cloud architectures constantly bridge legacy enterprise XML (ISO 20022 banking, HL7 healthcare, FIXM aviation) with modern JSON microservices and web apps.

PolyXML includes an **ultra-fast, zero-copy streaming transcoder** built directly into the Rust engine:

### 1. CLI Stdin / Stdout Streaming Pipes
```bash
# 1. Transcode XML to JSON with W3C XSD schema typing
cat order.xml | polyxml transcode --schema order.xsd --pretty > order.json

# 2. Transcode JSON back to XML with root element specification
cat order.json | polyxml transcode --schema order.xsd --root order --pretty > order.xml

# 3. Dynamic schema-less transcoding with automatic attribute (@attr) and text (#text) preservation
polyxml transcode legacy-feed.xml --out modern-feed.json
```

### 2. High-Performance Python Transcoding API
```python
import polyxml

# Direct C/Rust transcoding without intermediate DOM or Python loops:
json_bytes = polyxml.xml_to_json(xml_bytes, schema_path="order.xsd", indent=2)
xml_bytes = polyxml.json_to_xml(json_bytes, schema_path="order.xsd", root="order", indent=2)

# Or transcode through strongly-typed models:
json_bytes = polyxml.xml_to_json(xml_bytes, model=Order, indent=2)
xml_bytes = polyxml.json_to_xml(json_bytes, model=Order, indent=2)
```

---

## 🎯 Target Language Matrix

PolyXML strictly generates code adhering to modern programming paradigms (2024–2026), eliminating legacy boilerplate:

| Target Language | CLI Flag (`--lang`) | Generated Code Paradigm | Modern Features & Highlights |
| :--- | :--- | :--- | :--- |
| **Python 3.12+** | `python` | `@dataclass(slots=True)` & Pydantic v2 | PEP 695 type aliases, PEP 604 unions, restriction facets, native JSON codecs (10x faster xsdata replacement) |
| **Rust 2021/2024** | `rust` | Zero-copy `Cow<'a, str>` & Owned structs | Automatic Tarjan SCC recursive boxing (`Box<T>`), inherent streaming codecs, inherent JSON serialization |
| **C++20 / C++23** | `cpp` | Modern value types & `std::variant` | C++20 concepts, `std::unique_ptr` cycle breaks, CMake/Meson export, zero Xerces |
| **Java 21+** | `java` | Modern `record` & `sealed interface` | Exhaustive switch pattern matching, compact constructor facet validation, zero JNI |
| **TypeScript 5+** | `typescript` | Interfaces & Discriminated Unions | Runtime Zod schemas, circular reference resolution via `z.lazy()`, `as const` enums |
| **Go 1.22+** | `go` | Structs with `encoding/xml` & `encoding/json` | Dual `xml:"..."` and `json:"..."` struct tags, `json:"-"` on `XMLName`, custom `UnmarshalXML` choice exclusivity |
| **C# 12 / .NET 8+** | `csharp` | Records with Primary Constructors | Parameterless constructors, `[JsonPropertyName]` & `[JsonConverter]`, `IValidatableObject` |

👉 **[Read the Full Schema Compiler & CLI Guide →](docs/guides/compiler.md)**

---

## 🥊 Why PolyXML? (Old Way vs. PolyXML Way)

| Feature | Legacy Toolchains (JAXB, CodeSynthesis, xsdata, xgen) | PolyXML Modern Approach |
| :--- | :--- | :--- |
| **Compiler Architecture** | Fragmented language-specific scripts; unmaintained or closed-source | Single unified safe Rust compiler (like `protoc`), emitting 7 languages |
| **Parsing Performance** | Slow reflection or Python-level loops (**16x–38x slower**) | Zero-allocation Rust streaming engine (**51 MB/s** throughput) |
| **Micro-Telemetry Latency**| 43.0 μs per packet in Python (`xsdata`) | **2.5 μs** per packet (**17.1x speedup**, beating raw C DOM parsers) |
| **Memory Footprint** | Intermediate DOM node trees inflate RAM by **10x–20x** | Monomorphized event streaming, zero intermediate DOM allocation |
| **XML ↔ JSON Transcoding** | Brittle untyped dicts (`xmltodict`), slow Python loops, duplicate schemas | Zero-copy streaming CLI (`polyxml transcode`) & dual-format models across all targets |
| **Generated Code Quality**| Pre-C++11 raw pointers, mutable JavaBeans with getters/setters | Immutable Java 21+ records, modern C++20 value types, C# 12 records |
| **Licensing** | GPL v2 dual-licensing or per-seat commercial paywalls | **100% Permissive MIT License** (zero commercial royalties) |

👉 **[Read the Full In-Depth Architectural Comparison & Benchmark Breakdown →](docs/why-polyxml.md)**

---

## 🏛️ Architecture

PolyXML operates across two synchronized pipelines:

```
                          COMPILER PIPELINE (polyxml CLI)
  ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
  │ W3C XSD 1.0/1.1 │──────▶│  SchemaParser   │──────▶│   PolyXML-IR    │
  │  Schema Files   │       │   (Pure Rust)   │       │(Normalized AST) │
  └─────────────────┘       └─────────────────┘       └────────┬────────┘
                                                               │ Tarjan SCC Cycle Breaks
                                                               ▼
  ┌─────────────────────────────────────────────────────────────────────┐
  │                        7 Target Code Generators                     │
  │   Python  │  Rust  │  C++20  │  Java 21  │  TypeScript  │  Go  │ C# │
  └──────────────────────────────────┬──────────────────────────────────┘
                                     │ Generates typed models & codecs
                                     ▼
                          STREAMING RUNTIME PIPELINE
  ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
  │  Raw XML Stream │──────▶│  polyxml-core   │──────▶│  Typed In-Memory│
  │ (Files, Network)│       │quick-xml+lexical│       │     Objects     │
  └─────────────────┘       └─────────────────┘       └─────────────────┘
```

---

## 🚀 Performance Benchmarks

Measured on standard, reproducible workloads ([full methodology & reproduction steps](docs/benchmarks.md)).

### 1. Large Document Throughput (10,000 Catalog Items, 724 KB XML)

| Engine | Paradigm / Category | Implementation | Deserialization Latency | Deserialization Throughput | Serialization Latency | Peak RAM |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: |
| **PolyXML** | **Typed Dataclass** | **Rust + PyO3** | **13.9 ms** | **51.0 MB/s** | **7.30 ms** | **2.0 MB** |
| `lxml.etree` | Untyped DOM | C / Cython (`libxml2`) | 10.0 ms | 70.5 MB/s | — | <0.1 MB |
| `ElementTree` | Untyped DOM | Python Stdlib C/Python | 12.3 ms | 57.5 MB/s | — | 7.1 MB |
| `defusedxml` | Secure DOM | Python Defused | 27.2 ms | 26.0 MB/s | — | 7.1 MB |
| `xmltodict` | Untyped Dict | C (`pyexpat`) | 56.6 ms | 12.5 MB/s | 79.0 ms | 4.8 MB |
| `xsdata` | Typed Dataclass | Pure Python | 222.5 ms | 3.2 MB/s | 282.6 ms | 3.3 MB |

> - **16.0x faster deserialization** & **38.7x faster serialization** than `xsdata`.
> - **4.1x faster** than `xmltodict` while returning genuine typed dataclasses instead of untyped string dicts.
> - **3.5x lower RAM** than Python's standard library `xml.etree.ElementTree`.

### 2. Real-Time Micro-Telemetry (Sensor ~100B, Telemetry Commands)

| Engine | Category | Deserialization Latency | Serialization Latency | Speedup vs Pure Python |
| :--- | :--- | :---: | :---: | :---: |
| **PolyXML** | **Typed Dataclass** | **2.5 μs** | **1.4 μs** | **17.1x** |
| **PolyXML (Pydantic)** | **Typed Pydantic v2** | **3.1 μs** | **1.5 μs** | **13.7x** |
| `lxml.etree` | Untyped DOM | 3.1 μs | — | 13.7x |
| `ElementTree` | Untyped DOM | 4.9 μs | — | 8.7x |
| `xmltodict` | Untyped Dict | 10.3 μs | 14.9 μs | 4.2x |
| `xsdata` | Typed Dataclass | 43.0 μs | 45.0 μs | 1.0x (Ref) |

Critical telemetry commands and sensor packets deserialize in **2.5 microseconds**, beating even raw C-based DOM parsers (`lxml` at 3.1 μs).

### 3. Key-Value Database & Binary IPC (10,000 Entities in MDBX)

When caching parsed models in transactional key-value databases (`libmdbx`, `LMDB`, `RocksDB`) or transferring entity batches across multiprocessing workers, `polyxml.dumps_binary()` and `polyxml.loads_binary()` eliminate Python's single-threaded pickle bottlenecks:

| Storage Pipeline | Dumps Throughput | Dumps Latency | Loads Throughput | Payload Size | MDBX Write Speed |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`CloudPickle + LZ4` (Legacy)** | 20,609 ops/s | 48.5 μs | 49,322 ops/s | 547 B | 18,287 ops/s |
| `Pickle 5 + LZ4` (Stdlib C) | 71,954 ops/s | 13.9 μs | 50,092 ops/s | 539 B | — |
| **`PolyXML Binary + LZ4`** | **163,192 ops/s** | **6.1 μs** | **64,781 ops/s** | **252 B** | **58,781 ops/s** |
| **`PolyXML Binary (Direct, No LZ4)`** | **213,003 ops/s** | **4.7 μs** | **84,673 ops/s** | **327 B** | **63,236 ops/s** |

> - **7.9x faster serialization** and **3.2x faster transactional writes into real MDBX**.
> - **53.9% smaller storage footprint** (252 B vs 547 B per entity).

---

## 📊 W3C XML Schema Conformance Benchmark

PolyXML is continuously benchmarked against the official **W3C XML Schema 1.0 & 1.1 Test Suite (XSTS)** using our dedicated test harness repository, **[polyxml-w3c-tests](https://github.com/nth-bailey/polyxml-w3c-tests)**.

- **Schema Compilation**: **635 / 636 groups passed (99.8%)**
- **Instance Validation & Round-Trip**: **489 / 507 instances passed (96.4%)**
- Zero regressions across Sun Microsystems, Microsoft, and NIST test suites.

---

## 🛠️ CLI Workspace Manifest (`polyxml.toml`)

Manage multi-schema, multi-target enterprise projects with a declarative configuration file:

```toml
[workspace]
name = "enterprise-data-pipeline"
schemas = ["schemas/iso20022/*.xsd"]
include_dirs = ["schemas/common/"]
output_base_dir = "./generated"

[[generate]]
target = "python"
output = "src/generated/python"
backend = "pydantic-v2"
codecs = true

[[generate]]
target = "rust"
output = "src/generated/rust"
zero_copy = true
codecs = true

[[generate]]
target = "java"
output = "src/generated/java"
package = "com.enterprise.banking.iso20022"

[[generate]]
target = "typescript"
output = "src/generated/ts"
zod = true

[[generate]]
target = "cpp"
output = "src/generated/cpp"

[[generate]]
target = "go"
output = "src/generated/go"
package = "payments"

[[generate]]
target = "csharp"
output = "src/generated/csharp"
namespace = "Enterprise.Banking.Iso20022"
```

---

## Language Ecosystem & Packages

| Ecosystem / Language | Package / Registry | Installation | Interop Tech | Status |
| :--- | :--- | :--- | :--- | :---: |
| **Rust (Core)** | [![crates.io](https://img.shields.io/crates/v/polyxml.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml) | `cargo add polyxml` | Native Zero-Copy | 🟢 Stable |
| **Rust (CLI)** | [![crates.io](https://img.shields.io/crates/v/polyxml-cli.svg?logo=rust&label=polyxml-cli)](https://crates.io/crates/polyxml-cli) | `cargo install polyxml-cli` | Native CLI Compiler | 🟢 Stable |
| **Rust (C-ABI)** | [![crates.io](https://img.shields.io/crates/v/polyxml-c.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml-c) | `cargo add polyxml-c` | C-ABI Shared Lib | 🟢 Stable |
| **Python** | [![PyPI](https://img.shields.io/pypi/v/polyxml.svg?logo=pypi&label=PyPI)](https://pypi.org/project/polyxml/) | `pip install polyxml` | PyO3 (`abi3-py312`) | 🟢 Stable |
| **TypeScript / Node** | [![npm](https://img.shields.io/npm/v/polyxml.svg?logo=npm&color=CB3837&label=npm)](https://www.npmjs.com/package/polyxml) | `npm install polyxml` | `napi-rs` Native Addon | 🟢 Stable |
| **Java** | [![Maven Central](https://img.shields.io/maven-central/v/io.github.nth-bailey/polyxml.svg?logo=apache-maven&color=C71A36&label=Maven)](https://central.sonatype.com/artifact/io.github.nth-bailey/polyxml) | `<artifactId>polyxml</artifactId>` | Java 22+ Panama FFI | 🟢 Stable |
| **Go** | [![Go Reference](https://pkg.go.dev/badge/github.com/nth-bailey/PolyXML/bindings/go.svg)](https://pkg.go.dev/github.com/nth-bailey/PolyXML/bindings/go) | `go get github.com/nth-bailey/PolyXML/bindings/go` | Cgo (`polyxml.h`) | 🟢 Stable |
| **Modern C++20 / C** | [Conan](conan/) / [vcpkg](packaging/vcpkg/) (`polyxml`) | `conan install` / `vcpkg install polyxml` | Header-Only C++20 & Native Lib | 🟢 Stable |
| **C# / .NET 8+** | NuGet / Native | `dotnet add package PolyXML` | C# 12 Records & `System.Xml` | 🟢 Stable |
| **macOS & Linux** | [Homebrew Tap](https://github.com/nth-bailey/homebrew-polyxml) | `brew install nth-bailey/polyxml/polyxml` | Native Headers & Dynamic Lib | 🟢 Stable |

---

## Documentation & Learning

- **[Multi-Language Quickstart](https://nth-bailey.github.io/PolyXML/quickstart/)**: 5-minute setup across all 7 target ecosystems.
- **[Schema Compiler & CLI Guide](https://nth-bailey.github.io/PolyXML/guides/compiler/)**: Full reference for `polyxml generate`, `build`, `validate`, and `polyxml.toml`.
- **[Why PolyXML? Architectural Breakdown](https://nth-bailey.github.io/PolyXML/why-polyxml/)**: Deep comparison against JAXB, CodeSynthesis, xsdata, xgen, and xsd.exe.
- **[Architecture & Streaming Pipeline](https://nth-bailey.github.io/PolyXML/architecture/)**: Detailed breakdown of our zero-copy reader, frame stack, and Tarjan cycle-cutting.
- **[Performance Benchmarks](https://nth-bailey.github.io/PolyXML/benchmarks/)**: Reproducible benchmarks and throughput charts.

---

## License

Licensed under the [MIT License](LICENSE).
