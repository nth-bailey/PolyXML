---
title: PolyXML — High-Performance Polyglot XML Engine
description: Ultra-fast streaming XML data-binding engine in Rust with native bindings for Rust, Python, C++, Go, Java, and TypeScript.
---

# PolyXML

<p align="center">
  <strong>The Universal Native XML Data-Binding Engine</strong>
</p>

---

## Welcome to PolyXML

**PolyXML** is a high-performance native XML engine written in Rust for ultra-fast, streaming XML serialization and deserialization. It bridges raw XML directly to strongly-typed data structures across modern language runtimes with **zero unnecessary allocations**.

While web ecosystems shifted toward JSON and Protocol Buffers, mission-critical infrastructure in **defense & aerospace (UCI)**, **finance (ISO 20022, FIXML)**, and **healthcare (HL7)** remains deeply reliant on XML. PolyXML breaks down language barriers and eliminates legacy performance penalties by providing **one unified, native Rust engine for all tech stacks**.

---

## Supported Ecosystems

=== "Rust"
    Native zero-copy core engine via `polyxml-core` on [crates.io](https://crates.io/crates/polyxml-core). Monomorphized, fast streaming parser.

=== "Python"
    Accelerates Python `dataclasses` and **Pydantic v2** models via PyO3 (`abi3-py312`). 10x–30x faster than pure Python XML parsers.

=== "Modern C++20"
    Zero-overhead modern C++20 header-only wrapper (`polyxml.hpp`) with RAII memory management, designed for avionics, robotics, and defense.

=== "Go"
    High-throughput Cgo wrapper providing `polyxml.Unmarshal` and `polyxml.Marshal`, replacing Go's slow reflection-based `encoding/xml`.

=== "TypeScript & Node"
    Native Node.js addon compiled via `napi-rs` with full TypeScript definitions (`index.d.ts`), ideal for high-throughput microservices.

=== "Java (Panama FFI)"
    Java 22+ Foreign Function & Memory API (JEP 454) binding directly to off-heap memory with zero JNI boilerplate.

---

## Architecture at a Glance

```mermaid
flowchart TD
    XML[Raw XML Stream] --> CORE[polyxml-core<br/>quick-xml + lexical-core]
    CORE --> CABI[polyxml-c<br/>C-ABI Shared Library]
    CORE --> PY[polyxml-python<br/>PyO3 / abi3]
    CORE --> NAPI[polyxml-js<br/>napi-rs]

    CABI --> CPP[Modern C++20<br/>polyxml.hpp]
    CABI --> GO[Go<br/>Cgo]
    CABI --> JAVA[Java 22+<br/>Panama FFI]

    PY --> PYMODELS[Python Dataclasses<br/>& Pydantic v2]
    NAPI --> TS[Node.js / TypeScript]
```

---

## Next Steps

- Check out the [5-Minute Multi-Language Quickstart](quickstart.md) to see PolyXML in action.
- Read about our [Architecture & Streaming Design](architecture.md).
- Explore [Performance & Benchmarks](benchmarks.md).
