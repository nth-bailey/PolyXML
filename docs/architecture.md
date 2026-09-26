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

---

## 3. XSD-to-Code Generation & Intermediate Representation (IR)

In addition to runtime streaming data-binding, PolyXML includes a polyglot XSD-to-code generation engine inside `polyxml-core`:

```mermaid
flowchart TD
    subgraph Frontend [Pass 1 & 2: Parser]
        XSD[XSD 1.0 / 1.1 Documents] --> PARSER[Streaming XSD Parser]
        INC[Includes & Imports & Redefines] --> PARSER
    end

    subgraph IR [PolyXML-IR]
        PARSER --> SCHEMAS[SchemaIR<br/>StructDef, EnumDef, UnionDef, TypeAlias]
        SCHEMAS --> TOPO[3-Color Topological Sorter]
        SCHEMAS --> TARJAN[Tarjan SCC Cycle Detector]
    end

    subgraph Backend [Code Generators]
        TOPO --> CODEGEN[Target Codegen Engine]
        TARJAN -. Cycle Cuts (Box/Pointer/Lazy) .-> CODEGEN
        CODEGEN --> RS[Rust 2021/2024]
        CODEGEN --> PY[Python 3.12+]
        CODEGEN --> CPP[C++20/C++23]
        CODEGEN --> JV[Java 22+]
        CODEGEN --> TS[TypeScript 5+]
        CODEGEN --> GO[Go 1.22+]
        CODEGEN --> CS[C# 12 / .NET 8+]
    end
```

### Key Compilation Invariants
1. **Pure-Rust XSD Parser**: Ingests complex W3C schemas with full resolution of `include`, `import`, and `redefine` without external C libraries.
2. **Intermediate Representation (PolyXML-IR)**: Strips XML Schema idiosyncrasies and normalizes types into clean structs, enums, discriminated unions, and field metadata.
3. **Tarjan SCC Cycle-Cutting**: Detects recursive type loops at compile time and calculates minimal cut points, preventing recursive type infinite-size errors across target languages:
   - **Rust**: Inserts `Box<T>` or `Option<Box<T>>`.
   - **Go**: Inserts pointer types (`*T`).
   - **C++**: Inserts `std::unique_ptr<T>`.
   - **TypeScript**: Emits recursive `z.lazy(() => ...)` wrappers in Zod schemas.
4. **Codecs Synthesis**: Automatically generates streaming XML serialization and deserialization methods directly within emitted data models for maximum performance.

