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
| **Python Backend** | `--backend` | Python model style: `dataclass` or `pydantic-v2`. | `dataclass` |
| **Rust Zero-Copy** | `--zero-copy` | Use `Cow<'a, str>` string slices instead of owned `String`. | `true` |
| **Streaming Codecs**| `--codecs` | Emit inherent zero-copy streaming XML serializers and deserializers. | `true` |
| **Zod Schemas** | `--zod` | Emit runtime Zod validation schemas for TypeScript. | `false` |
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

## 🎯 Target Language Matrix

| Target | Language Version | Paradigm | Key Highlights |
|---|---|---|---|
| **Python** | Python 3.12+ | `@dataclass` & Pydantic v2 | PEP 695 type aliases, PEP 604 unions, zero-copy streaming codecs |
| **Rust** | Rust 2021 / 2024 | Zero-Copy & Owned Structs | Lifetime inference `<'a>`, automatic `Box<T>` cycle breaks, inherent streaming codecs |
| **C++** | C++20 / C++23 | Header-Only Value Types | `std::variant` choices, `std::unique_ptr` cycle breaks, C++20 concepts, CMake/Meson export |
| **Java** | Java 21+ | Modern Records & Sealed Interfaces | Exhaustive switch pattern matching, compact constructor facet validation |
| **TypeScript** | TypeScript 5+ | Interfaces & Discriminated Unions | Runtime Zod schemas, circular references handled via `z.lazy()` |
| **Go** | Go 1.22+ | Structs with `encoding/xml` | Canonical initialism normalization (`ID`, `URL`), pointer cycle cuts, choice mutual exclusivity |
| **C#** | C# 12 / .NET 8+ | Records with Primary Constructors | Parameterless constructors for `XmlSerializer`, polymorphic choice records, `IValidatableObject` |

---

## 📊 W3C XML Schema Conformance

PolyXML's schema compiler and runtime codecs are continuously tested against the official **W3C XML Schema 1.0 / 1.1 Test Suite (XSTS)** using our dedicated test harness repository, **[polyxml-w3c-tests](https://github.com/nth-bailey/polyxml-w3c-tests)**.

- **Schema Compilation Pass Rate**: **99.8% (635 / 636 groups)**
- **Instance Validation & Round-Trip Pass Rate**: **96.4% (489 / 507 instances)**

For full conformance benchmark metrics across Sun Microsystems, Microsoft, and NIST test sets, visit the [polyxml-w3c-tests repository](https://github.com/nth-bailey/polyxml-w3c-tests).
