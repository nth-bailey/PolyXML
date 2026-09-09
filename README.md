# PolyXML

<p align="center">
  <strong>The High-Performance, Polyglot XML Data-Binding Engine</strong>
</p>

<p align="center">
  <a href="https://github.com/nth-bailey/PolyXML/actions"><img src="https://img.shields.io/github/actions/workflow/status/nth-bailey/PolyXML/ci.yml?branch=main&label=CI&logo=github" alt="CI"></a>
  <a href="https://nth-bailey.github.io/PolyXML/"><img src="https://img.shields.io/badge/docs-zensical-blue.svg?logo=gitbook" alt="Docs"></a>
  <a href="https://github.com/nth-bailey/PolyXML/blob/main/crates/polyxml-python/pyproject.toml"><img src="https://img.shields.io/badge/coverage-100%25-brightgreen.svg?logo=pytest" alt="Coverage: 100%"></a>
  <a href="https://github.com/astral-sh/ruff"><img src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json" alt="Ruff"></a>
  <a href="https://github.com/pre-commit/pre-commit"><img src="https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white" alt="pre-commit"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/polyxml"><img src="https://img.shields.io/crates/v/polyxml.svg?logo=rust&label=crates.io" alt="crates.io: polyxml"></a>
  <a href="https://crates.io/crates/polyxml-c"><img src="https://img.shields.io/crates/v/polyxml-c.svg?logo=rust&label=polyxml-c" alt="crates.io: polyxml-c"></a>
  <a href="https://pypi.org/project/polyxml/"><img src="https://img.shields.io/pypi/v/polyxml.svg?logo=pypi&label=PyPI" alt="PyPI: polyxml"></a>
  <a href="https://www.npmjs.com/package/polyxml"><img src="https://img.shields.io/npm/v/polyxml.svg?logo=npm&color=CB3837&label=npm" alt="npm: polyxml"></a>
  <a href="https://central.sonatype.com/artifact/io.github.nth-bailey/polyxml"><img src="https://img.shields.io/maven-central/v/io.github.nth-bailey/polyxml.svg?logo=apache-maven&color=C71A36&label=Maven" alt="Maven Central"></a>
  <a href="https://pkg.go.dev/github.com/nth-bailey/PolyXML/bindings/go"><img src="https://pkg.go.dev/badge/github.com/nth-bailey/PolyXML/bindings/go.svg" alt="Go Reference"></a>
  <a href="https://github.com/nth-bailey/homebrew-polyxml"><img src="https://img.shields.io/badge/Homebrew-polyxml-FBB040.svg?logo=homebrew&logoColor=black" alt="Homebrew"></a>
  <a href="https://github.com/conan-io/conan-center-index/pull/30940"><img src="https://img.shields.io/badge/Conan-PR%20%2330940-004B87.svg?logo=conan" alt="ConanCenter PR"></a>
  <a href="https://github.com/conda-forge/staged-recipes/pull/34791"><img src="https://img.shields.io/badge/conda--forge-PR%20%2334791-000000.svg?logo=conda-forge" alt="conda-forge PR"></a>
  <a href="https://github.com/microsoft/vcpkg/pull/53866"><img src="https://img.shields.io/badge/vcpkg-PR%20%2353866-5C2D91.svg?logo=microsoft" alt="vcpkg PR"></a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust" alt="Rust: 1.80+"></a>
  <a href="https://www.python.org"><img src="https://img.shields.io/badge/Python-3.12%20%7C%203.13-3776AB.svg?logo=python&logoColor=white" alt="Python: 3.12 | 3.13"></a>
  <a href="https://nodejs.org"><img src="https://img.shields.io/badge/Node.js-20%20%7C%2022-339933.svg?logo=node.js&logoColor=white" alt="Node.js: 20 | 22"></a>
  <a href="https://openjdk.org/projects/panama/"><img src="https://img.shields.io/badge/Java-22%2B%20Panama-ED8B00.svg?logo=openjdk&logoColor=white" alt="Java: 22+ Panama"></a>
  <a href="https://go.dev"><img src="https://img.shields.io/badge/Go-1.22%2B-00ADD8.svg?logo=go&logoColor=white" alt="Go: 1.22+"></a>
  <a href="https://en.cppreference.com/w/cpp/20"><img src="https://img.shields.io/badge/C%2B%2B-20-00599C.svg?logo=c%2B%2B" alt="C++: 20"></a>
</p>

---

## Overview

**PolyXML** is a universal native XML engine engineered in Rust for ultra-fast, streaming XML serialization and deserialization. It bridges raw XML directly to strongly-typed data structures across modern language runtimes with **zero unnecessary allocations**.

While modern web ecosystems shifted to JSON and Protocol Buffers, mission-critical industries—including **defense & aerospace (UCI)**, **finance (ISO 20022, FIXML)**, and **healthcare (HL7)**—continue to rely on XML. PolyXML eliminates the single-language silos and performance penalties of legacy XML data-binding tools by providing one optimized, native Rust core for all stacks.

---

## Architecture

```
                  ┌─────────────────────────────────────┐
                  │          Raw XML Stream             │
                  └──────────────────┬──────────────────┘
                                     │
                                     ▼
                  ┌─────────────────────────────────────┐
                  │            polyxml-core             │
                  │   - quick-xml event reader/writer   │
                  │   - lexical-core scalar parser      │
                  │   - Language-agnostic Schema IR     │
                  │   - Zero-copy streaming state       │
                  └─────────┬───────────────┬───────────┘
                            │               │
           ┌────────────────┼───────────────┼────────────────┐
           ▼                ▼               ▼                ▼
    ┌──────────────┐ ┌─────────────┐ ┌─────────────┐  ┌─────────────┐
    │  Rust / Core │ │   Python    │ │  C++ & Go   │  │ Node / Wasm │
    │ Direct Crate │ │ (PyO3 abi3) │ │(C-ABI / Cgo)│  │  (napi-rs)  │
    └──────────────┘ └─────────────┘ └─────────────┘  └─────────────┘
```

---

## Key Features

- **⚡ Blazing Fast**: Powered by `quick-xml` streaming event loop and `lexical-core` byte-slice parsing. Zero DOM intermediate allocations.
- **🌊 Streaming Iterator**: Parse multi-gigabyte XML documents with $O(1)$ constant memory (<5 MB RAM) via `polyxml.iterparse()`.
- **🔄 Bidirectional**: Full support for both **deserialization** (XML $\to$ typed models) and **serialization** (typed models $\to$ XML).
- **🌐 Polyglot by Design**: The core engine is 100% pure Rust with zero Python or language runtime dependencies, ready to be embedded anywhere.
- **🎯 Full Schema Support**: Namespaces, attributes vs. elements, text nodes, `xsi:nil`, choice, lists, and ISO-8601 date/time scalar types.

---

## 🚀 Performance & Benchmarks

PolyXML is benchmarked against the Python and native XML ecosystems on standard, reproducible workloads ([full methodology & data](docs/benchmarks.md)).

### 1. Large Document Throughput (10,000 Catalog Items, 724 KB XML)

| Engine | Paradigm / Category | Implementation | Deserialization Latency | Deserialization Throughput | Serialization Latency | Peak RAM |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: |
| **PolyXML** | **Typed Dataclass** | **Rust + PyO3** | **13.9 ms** | **51.0 MB/s** | **7.30 ms** | **2.0 MB** |
| `lxml.etree` | Untyped DOM | C / Cython (`libxml2`) | 10.0 ms | 70.5 MB/s | — | <0.1 MB |
| `ElementTree` | Untyped DOM | Python Stdlib C/Python | 12.3 ms | 57.5 MB/s | — | 7.1 MB |
| `defusedxml` | Secure DOM | Python Defused | 27.2 ms | 26.0 MB/s | — | 7.1 MB |
| `xmltodict` | Untyped Dict | C (`pyexpat`) | 56.6 ms | 12.5 MB/s | 79.0 ms | 4.8 MB |
| `xsdata` | Typed Dataclass | Pure Python | 222.5 ms | 3.2 MB/s | 282.6 ms | 3.3 MB |

> - **16.0x faster** deserialization & **38.7x faster** serialization than `xsdata` (standard typed dataclasses).
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

Critical telemetry commands and sensor packets deserialize in **2.5 microseconds**, beating even C-based DOM parsers (`lxml` at 3.1 μs).

---

## Language Ecosystem & Packages

| Ecosystem / Language | Package / Registry | Installation | Interop Tech | Status |
| :--- | :--- | :--- | :--- | :---: |
| **Rust (Core)** | [![crates.io](https://img.shields.io/crates/v/polyxml.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml) | `cargo add polyxml` | Native Zero-Copy | 🟢 Stable |
| **Rust (C-ABI)** | [![crates.io](https://img.shields.io/crates/v/polyxml-c.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml-c) | `cargo add polyxml-c` | C-ABI Shared Lib | 🟢 Stable |
| **Python** | [![PyPI](https://img.shields.io/pypi/v/polyxml.svg?logo=pypi&label=PyPI)](https://pypi.org/project/polyxml/) | `pip install polyxml` | PyO3 (`abi3-py312`) | 🟢 Stable |
| **TypeScript / Node** | [![npm](https://img.shields.io/npm/v/polyxml.svg?logo=npm&color=CB3837&label=npm)](https://www.npmjs.com/package/polyxml) | `npm install polyxml` | `napi-rs` Native Addon | 🟢 Stable |
| **Java** | [![Maven Central](https://img.shields.io/maven-central/v/io.github.nth-bailey/polyxml.svg?logo=apache-maven&color=C71A36&label=Maven)](https://central.sonatype.com/artifact/io.github.nth-bailey/polyxml) | `<artifactId>polyxml</artifactId>` | Java 22+ Panama FFI | 🟢 Stable |
| **Go** | [![Go Reference](https://pkg.go.dev/badge/github.com/nth-bailey/PolyXML/bindings/go.svg)](https://pkg.go.dev/github.com/nth-bailey/PolyXML/bindings/go) | `go get github.com/nth-bailey/PolyXML/bindings/go` | Cgo (`polyxml.h`) | 🟢 Stable |
| **Modern C++20 / C** | [Conan](conan/) / [vcpkg](packaging/vcpkg/) (`polyxml`) | `conan install` / `vcpkg install polyxml` | Header-Only C++20 & Native Lib | 🟢 Stable |
| **macOS & Linux** | [Homebrew Tap](https://github.com/nth-bailey/homebrew-polyxml) | `brew install nth-bailey/polyxml/polyxml` | Native Headers & Dynamic Lib | 🟢 Stable |

---

## Quickstart Examples

### Rust
```rust
use std::sync::Arc;
use polyxml::schema::{ModelSchema, FieldSchema, FieldKind, ScalarType, ValueType};
use polyxml::deserialize;

let schema = ModelSchema::builder("User")
    .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
    .field(FieldSchema::new("name", b"name", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
    .build();

let xml = br#"<User id="42"><name>Alice</name></User>"#;
let value = deserialize(xml, Arc::clone(&schema))?;
```

### Python
```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Item:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    price: float = field(metadata={"type": "Element"})

# 1. Deserialize full XML into a typed Python dataclass
item = polyxml.deserialize(b'<Item id="1"><name>Turbine</name><price>99.5</price></Item>', Item)

# 2. Stream huge XML documents with O(1) constant memory (<5 MB RAM)
for item in polyxml.iterparse(open("large_catalog.xml", "rb").read(), Item, tag="Item"):
    print(item.name, item.price)

# 3. Serialize model back to XML
xml_bytes = polyxml.serialize(item, indent=2)
```

### Modern C++20
```cpp
#include "polyxml.hpp"

auto schema = polyxml::SchemaBuilder("Sensor")
    .add_attribute("id", "id", POLYXML_SCALAR_INT)
    .add_element("name", "name", POLYXML_SCALAR_STRING)
    .build();

auto val = polyxml::deserialize(R"(<Sensor id="101"><name>Gyro</name></Sensor>)", schema);
std::string name = val.get("name")->as_string().value();
```

### Go
```go
import "github.com/nth-bailey/PolyXML/bindings/go"

builder, _ := polyxml.NewSchemaBuilder("Device")
builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
builder.AddField("name", "name", polyxml.FieldElement, polyxml.ScalarString)
schema, _ := builder.Build()

val, err := polyxml.Deserialize(xmlBytes, schema)
name, _ := val.GetField("name").GetString()
```

### TypeScript / Node.js
```typescript
import { deserialize, serialize } from 'polyxml';

const schema = {
  name: 'Item',
  fields: [
    { name: 'id', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
    { name: 'name', xmlName: 'name', kind: 'element', scalarType: 'string' }
  ]
};

const obj = deserialize('<Item id="7"><name>Motor</name></Item>', schema);
```

### Java 22+ (Project Panama FFI)
```java
import io.polyxml.PolyXML;

try (var schema = new PolyXML.SchemaBuilder("Sensor")
        .addField("id", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
        .addField("name", "name", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING)
        .build()) {

    System.out.println("PolyXML Native Version: " + PolyXML.version());
}
```

---

## Repository Structure

```
PolyXML/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── polyxml-core/           # Pure Rust core streaming engine
│   ├── polyxml-python/         # Python bindings (PyO3 + Maturin)
│   ├── polyxml-c/              # Universal C-ABI shared library + polyxml.h
│   └── polyxml-js/             # Node.js & TypeScript bindings (napi-rs)
├── bindings/
│   ├── cpp/                    # Header-only modern C++20 wrapper (polyxml.hpp)
│   ├── go/                     # Go package using Cgo (polyxml.go)
│   └── java/                   # Java 22+ Project Panama FFI (PolyXML.java)
```

---

## License

Licensed under the [MIT License](LICENSE).
