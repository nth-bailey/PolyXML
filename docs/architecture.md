---
title: Architecture & Design
description: Inside PolyXML's high-performance streaming engine and universal C-ABI bridge.
---

# Architecture & Design

PolyXML was designed from the ground up to solve two fundamental problems in software engineering:

1. **The Language Silo Problem**: XML data binding has historically required every programming language to reinvent its own parser from scratch (e.g. `xsdata` in Python, `encoding/xml` in Go, JAXB in Java, CodeSynthesis in C++).
2. **The Allocation Bottleneck**: Traditional XML libraries allocate intermediate Document Object Model (DOM) node trees, creating severe memory churn and latency spikes.

---

## 1. Zero-Allocation Streaming Pipeline

PolyXML processes XML byte streams using a state-machine reader powered by `quick-xml`:

```mermaid
sequenceDiagram
    participant Raw as Raw XML Byte Buffer
    participant Reader as quick-xml Streaming Reader
    participant Stack as Frame Stack
    participant Lex as lexical-core Parser
    participant Target as Target Model (Host Language)

    Raw->>Reader: read_event_into(&mut buf)
    Reader->>Stack: Event::Start (Push frame)
    Reader->>Lex: Parse attributes from byte slices
    Lex-->>Stack: Insert scalar values
    Reader->>Stack: Event::Text (Accumulate text buf)
    Reader->>Reader: Event::End (Pop frame)
    Stack->>Target: Instantiate target object directly
```

### Key Engineering Invariants
- **Zero Intermediate DOM Allocation**: Tags and attributes are matched against the pre-compiled `ModelSchema` hash tables on the fly.
- **Fast Numeric Conversions**: Integers and floating-point numbers are converted directly from ASCII byte slices using `lexical-core` without intermediate UTF-8 heap string allocations.
- **Memory Reuse**: A single reusable byte vector buffer is passed to `read_event_into`, avoiding heap churn on large documents.

---

## 2. Universal Polyglot Bridge

Rather than writing bespoke C extensions for each target language, PolyXML adopts a tiered FFI architecture:

```mermaid
graph TD
    A[polyxml-core <br/>Pure Rust] --> B[polyxml-c <br/>C-ABI Shared Library]
    A --> C[polyxml-python <br/>PyO3 / Python 3.12+ ABI3]
    A --> D[polyxml-js <br/>napi-rs Node Addon]

    B --> E[C++20 polyxml.hpp]
    B --> F[Go Cgo Package]
    B --> G[Java 22+ Panama FFI]
```

### Memory Management Across Language Boundaries

| Language | Memory Strategy | Overhead |
| :--- | :--- | :--- |
| **Rust** | Direct stack/heap ownership via RAII. | Zero |
| **C++** | `polyxml::Value` wraps native handles with RAII destructors. | Zero |
| **Python** | PyO3 allocates Python heap objects directly during frame completion. | Low |
| **Go** | Cgo allocates values off-heap; GC finalizers (`runtime.SetFinalizer`) free native memory. | Low |
| **Node.js** | NAPI converts `PolyValue` directly into V8 JavaScript heap objects. | Low |
| **Java** | Project Panama allocates and accesses off-heap memory via `Arena.ofConfined()`. | Zero JNI Overhead |
