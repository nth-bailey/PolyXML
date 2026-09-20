---
title: Schema Compiler & CLI Toolchain
description: Compile W3C XML Schema 1.0 and 1.1 into type-safe models and codecs across 7 programming languages using polyxml.
---

# Schema Compiler & CLI Toolchain (`polyxml`)

PolyXML includes a high-performance, polyglot schema compiler and CLI toolchain (`polyxml`) that parses W3C XSD 1.0 and 1.1 schemas, builds a language-agnostic Intermediate Representation (PolyXML-IR), resolves complex type cycles via Tarjan's Strongly Connected Components (SCC) algorithm, and emits idiomatic, production-ready data contracts and codecs across **7 modern programming languages**.

---

## 🚀 Key Features

- **Pure-Rust XSD 1.0 & 1.1 Parser**: Zero dependencies on legacy libraries like `libxml2` or Apache Xerces.
- **PolyXML Intermediate Representation (IR)**: Normalized schema representation preserving namespaces, facets, substitution groups, documentation, and cardinality.
- **Tarjan SCC Cycle-Cutting**: Automatically identifies self-referential and mutually recursive types, calculating the minimal set of cycle-cut points to prevent infinite size allocations (`Box<T>`, pointers, `std::unique_ptr`, `z.lazy`).
- **Simultaneous Multi-Target Compilation**: Emit models for Python, Rust, C++, Java, TypeScript, Go, and C# in a single compiler invocation.
- **W3C Conformance Tested**: Validated against the official W3C XML Schema Test Suite (XSTS) via the [polyxml-w3c-tests](https://github.com/nth-bailey/polyxml-w3c-tests) harness.

---

## 📦 Installation

Compile and install the `polyxml` CLI directly from source using Cargo:

```bash
# From the PolyXML workspace root
cargo install --path crates/polyxml-cli
```

Verify installation:

```bash
polyxml --help
```

---

## 🛠️ CLI Commands

### 1. `polyxml generate`

Generate code directly from one or more `.xsd` schema files:

```bash
# Generate Python dataclasses
polyxml generate --lang python --out ./generated/python schema.xsd

# Generate Pydantic v2 models with runtime facet validation
polyxml generate --lang python --backend pydantic-v2 --out ./generated/python schema.xsd

# Generate zero-copy Rust models with inherent streaming codecs
polyxml generate --lang rust --zero-copy --codecs --out ./generated/rust schema.xsd

# Generate all 7 languages simultaneously
polyxml generate \
  --lang python \
  --lang rust \
  --lang cpp \
  --lang java \
  --lang typescript \
  --lang go \
  --lang csharp \
  --out ./generated \
  schema.xsd
```

#### Flags and Options

| Option | Flag | Description | Default |
|---|---|---|---|
| **Target Language** | `-l`, `--lang` | Target language (`python`, `rust`, `cpp`, `java`, `typescript`, `go`, `csharp`). Can be specified multiple times. | Required |
| **Output Directory** | `-o`, `--out` | Target directory for generated source files. | `.` |
| **Target Backend** | `-b`, `--backend` | Target backend (`dataclass`/`pydantic-v2` for Python; `standard`/`jackson` for Java; `standard`/`glaze` for C++; `none`/`zod`/`valibot`/`typebox` for TypeScript; `standard`/`easyjson`/`sonic` for Go). | Target default |
| **Compilation Mode** | `-m`, `--mode` | Target packaging mode (`header` or `modules` for C++). | Target default |
| **Rust Zero-Copy** | `--zero-copy` | Use `Cow<'a, str>` string slices instead of owned `String`. | `true` |
| **Rust rkyv** | `--rkyv` | Derive `rkyv::{Archive, Serialize, Deserialize}` for zero-copy wire format serialization. | `false` |
| **Streaming Codecs**| `--codecs` | Emit inherent zero-copy streaming XML serializers and deserializers. | `true` |
| **Validation Schemas** | `--zod` | Emit runtime Zod validation schemas for TypeScript (equivalent to `--backend zod`). | `false` |
| **C# Source-Gen** | `--source-gen` | Emit Native AOT compile-time `JsonSerializerContext` for C#. | `false` |
| **C# Record Kind** | `--record-kind` | C# record representation (`class` or `struct`). | `class` |
| **Package / Namespace** | `-p`, `--package` | Namespace or package name for Java, Go, C#, or C++. | Target default |
| **Dry Run** | `--dry-run` | Parse and print generated output without writing to disk. | `false` |
| **Format** | `--format` | Automatically format generated code using host toolchains (`ruff`, `cargo fmt`, `clang-format`, `gofmt`). | `true` |

---

### 2. `polyxml build`

Declaratively compile complex, multi-schema enterprise projects using a workspace manifest (`polyxml.toml`):

```bash
polyxml build --config polyxml.toml
```

#### Workspace Manifest Example (`polyxml.toml`)

```toml
[workspace]
name = "enterprise-iso20022"
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
rkyv = true

[[generate]]
target = "java"
output = "src/generated/java"
package = "com.enterprise.banking.iso20022"
backend = "jackson"

[[generate]]
target = "typescript"
output = "src/generated/ts"
backend = "valibot"

[[generate]]
target = "cpp"
output = "src/generated/cpp"
mode = "modules"
backend = "glaze"

[[generate]]
target = "go"
output = "src/generated/go"
package = "payments"
backend = "sonic"

[[generate]]
target = "csharp"
output = "src/generated/csharp"
namespace = "Enterprise.Banking.Iso20022"
source_gen = true
record_kind = "struct"
```

---

### 3. `polyxml validate`

Statically check W3C XML schemas for structural validity, element types, and cycle topology without emitting code:

```bash
polyxml validate schemas/*.xsd
```

Validates:
- XML syntax and W3C XSD element structure.
- Type reference integrity and namespace imports.
- Strongly Connected Components and recursion depth.

---

### 4. `polyxml transcode`

Bidirectionally transcode between XML and JSON using zero-copy streaming, with optional schema guidance:

```bash
# 1. Transcode XML to JSON with W3C XSD schema typing
polyxml transcode --schema order.xsd --pretty order.xml --out order.json

# 2. Stream directly through stdin / stdout pipes
cat order.xml | polyxml transcode --schema order.xsd > order.json

# 3. Transcode JSON back to XML with specified root element
cat order.json | polyxml transcode --schema order.xsd --root order --pretty > order.xml

# 4. Dynamic schema-less transcoding with attribute (@attr) and text (#text) preservation
polyxml transcode legacy.xml --out modern.json
```

#### Flags and Options

| Option | Flag | Description | Default |
|---|---|---|---|
| **Input** | `[INPUT]` | Input file path, or `-` / omitted for standard input. | `stdin` |
| **Output** | `-o`, `--out` | Output file path, or `-` / omitted for standard output. | `stdout` |
| **From Format** | `--from` | Input format (`xml` or `json`). Auto-detected if omitted. | Auto-detect |
| **To Format** | `--to` | Output format (`xml` or `json`). Auto-detected if omitted. | Auto-detect |
| **Schema** | `-s`, `--schema` | Optional W3C XSD schema file for typed schema-directed transcoding. | None |
| **Root Element**| `-r`, `--root` | Root XML element tag name (used when transcoding JSON to XML). | None |
| **Pretty** | `--pretty` | Format output with indentation and newlines. | `false` |

---

## 🎯 Target Language Matrix

| Target | Language Version | Paradigm | Key Highlights |
|---|---|---|---|
| **Python** | Python 3.12+ | `@dataclass` & Pydantic v2 | PEP 695 type aliases, PEP 604 unions, zero-copy streaming codecs, native `.to_json()` / `.from_json()` methods |
| **Rust** | Rust 2021 / 2024 | Zero-Copy & Owned Structs | Lifetime inference `<'a>`, automatic `Box<T>` cycle breaks, inherent streaming XML codecs, inherent `.to_json_string()` codecs |
| **C++** | C++20 / C++23 | Header-Only Value Types | `std::variant` choices, `std::unique_ptr` cycle breaks, C++20 concepts, CMake/Meson export |
| **Java** | Java 21+ | Modern Records & Sealed Interfaces | Exhaustive switch pattern matching, compact constructor facet validation |
| **TypeScript** | TypeScript 5+ | Interfaces & Discriminated Unions | Runtime Zod schemas, circular references handled via `z.lazy()`, `as const` enums |
| **Go** | Go 1.22+ | Structs with `encoding/xml` & `encoding/json` | Dual `xml:"..."` and `json:"..."` struct tags, `json:"-"` on `XMLName`, canonical initialisms (`ID`, `URL`), choice exclusivity |
| **C#** | C# 12 / .NET 8+ | Records with Primary Constructors | Parameterless constructors, dual `XmlSerializer` and `System.Text.Json` attributes (`[JsonPropertyName]`, `[JsonConverter]`), `IValidatableObject` |

---

## 📊 W3C XML Schema Conformance

PolyXML's schema compiler and runtime codecs are continuously tested against the official **W3C XML Schema 1.0 / 1.1 Test Suite (XSTS)** using our dedicated test harness repository, **[polyxml-w3c-tests](https://github.com/nth-bailey/polyxml-w3c-tests)**.

- **Schema Compilation Pass Rate**: **99.8% (635 / 636 groups)**
- **Instance Validation & Round-Trip Pass Rate**: **96.4% (489 / 507 instances)**

For full conformance benchmark metrics across Sun Microsystems, Microsoft, and NIST test sets, visit the [polyxml-w3c-tests repository](https://github.com/nth-bailey/polyxml-w3c-tests).

---

## 🌐 Real-World Enterprise Repositories

Explore full-scale enterprise examples demonstrating `polyxml.toml` manifests and multi-target compilation in real-world production settings:

- **[🛸 Defense & Aerospace Showcase (polyxml-defense-examples)](https://github.com/nth-bailey/polyxml-defense-examples)**:
  Compiles the **USAF UCI v2.5** schema standard and bridges edge sensor telemetry from **Anduril Lattice** across all 7 languages.
- **[💳 FinTech & Banking Showcase (polyxml-finance-examples)](https://github.com/nth-bailey/polyxml-finance-examples)**:
  Compiles the **ISO 20022 `pacs.008`** schema standard and bridges instant payment webhooks (FedNow, Stripe, Plaid) across all 7 languages.
- **[🚍 Smart Cities & Transit Showcase (polyxml-transit-examples)](https://github.com/nth-bailey/polyxml-transit-examples)**:
  Compiles the European **CEN SIRI v2.0** and **NeTEx** schemas and bridges live Google GTFS-Realtime feeds across all 7 languages.


