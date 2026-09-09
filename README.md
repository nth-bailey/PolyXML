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
  <a href="https://github.com/nth-bailey/PolyXML/releases"><img src="https://img.shields.io/github/v/release/nth-bailey/PolyXML?logo=github&color=333333&label=Release" alt="GitHub Release"></a>
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
- **🔄 Bidirectional**: Full support for both **deserialization** (XML $\to$ typed models) and **serialization** (typed models $\to$ XML).
- **🌐 Polyglot by Design**: The core engine is 100% pure Rust with zero Python or language runtime dependencies, ready to be embedded anywhere.
- **🎯 Full Schema Support**: Namespaces, attributes vs. elements, text nodes, `xsi:nil`, choice, lists, and ISO-8601 date/time scalar types.

---

## Language Ecosystem & Packages

| Ecosystem / Language | Package / Registry | Installation | Interop Tech | Status |
| :--- | :--- | :--- | :--- | :---: |
| **Rust (Core)** | [![crates.io](https://img.shields.io/crates/v/polyxml.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml) | `cargo add polyxml` | Native Zero-Copy | 🟢 Stable |
| **Rust (C-ABI)** | [![crates.io](https://img.shields.io/crates/v/polyxml-c.svg?logo=rust&label=crates.io)](https://crates.io/crates/polyxml-c) | `cargo add polyxml-c` | C-ABI Shared Lib | 🟢 Stable |
| **Python** | [![PyPI](https://img.shields.io/pypi/v/polyxml.svg?logo=pypi&label=PyPI)](https://pypi.org/project/polyxml/) | `pip install polyxml` | PyO3 (`abi3-py312`) | 🟢 Stable |
| **TypeScript / Node** | [![npm](https://img.shields.io/npm/v/polyxml.svg?logo=npm&color=CB3837&label=npm)](https://www.npmjs.com/package/polyxml) | `npm install polyxml` | `napi-rs` Native Addon | 🟢 Stable |
| **Java** | [![Maven Central](https://img.shields.io/maven-central/v/io.github.nth-bailey/polyxml.svg?logo=apache-maven&color=C71A36&label=Maven)](https://central.sonatype.com/artifact/io.github.nth-bailey/polyxml) | `<artifactId>polyxml</artifactId>` | Java 22+ Panama FFI | 🟢 Stable |
| **Go** | `github.com/nth-bailey/PolyXML/bindings/go` | `go get github.com/nth-bailey/PolyXML/bindings/go` | Cgo (`polyxml.h`) | 🟢 Stable |
| **Modern C++20** | `bindings/cpp` (`polyxml_cpp`) | CMake `target_link_libraries` | Header-Only C++20 | 🟢 Stable |

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

# Deserialize XML into Python dataclass
item = polyxml.deserialize(b'<Item id="1"><name>Turbine</name><price>99.5</price></Item>', Item)

# Serialize model back to XML
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
