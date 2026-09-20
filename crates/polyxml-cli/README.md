# PolyXML CLI (`polyxml`)

Unified developer command-line interface and polyglot schema compiler toolchain for PolyXML.

`polyxml` parses W3C XSD 1.0 and 1.1 schemas into a unified, language-agnostic Intermediate Representation (IR), resolves cyclic/recursive types via Tarjan's Strongly Connected Components (SCC) algorithm, and compiles production-ready, idiomatic data models and codecs for 7 target languages.

---

## Installation

```bash
# From workspace root
cargo install --path crates/polyxml-cli
```

---

## Supported Target Languages

| Target | Flag (`--lang`) | Generated Artifacts & Features |
| :--- | :--- | :--- |
| **Python** | `python` | Modern Python 3.12+ `@dataclass` or Pydantic v2 models, field constraints, zero-copy streaming codecs |
| **Rust** | `rust` | Zero-copy `Cow<'a, str>` & owned structs, automatic recursive boxing (`Box<T>`), streaming serializers/deserializers, rkyv wire format (`--rkyv`) |
| **C++** | `cpp` | Modern C++20/C++23 value types, C++20 Modules (`--mode modules`), Glaze reflection (`--backend glaze`), CMake/Meson export |
| **Java** | `java` | Java 21+ records, `sealed interface` choice models, Jackson XML/JSON annotations (`--backend jackson`) |
| **TypeScript** | `typescript` | TypeScript 5+ interfaces, discriminated unions, runtime validation schemas via Zod, Valibot, or TypeBox (`--backend`) |
| **Go** | `go` | Idiomatic Go 1.22+ structs with `encoding/xml` tags, reflectionless EasyJSON (`--backend easyjson`) & ByteDance Sonic JIT (`--backend sonic`) |
| **C#** | `csharp` | Modern C# 12 / .NET 8+ records and record structs (`--record-kind struct`), compile-time Native AOT source generation (`--source-gen`) |

---

## Commands

### 1. `polyxml generate`

Compile schemas directly into code for one or more target languages:

```bash
# Generate Python dataclasses
polyxml generate --lang python --out ./generated/python schemas/order.xsd

# Generate Pydantic v2 models with runtime validation
polyxml generate --lang python --backend pydantic-v2 --out ./generated/python schemas/order.xsd

# Generate Java 21 records with Enterprise Jackson annotations
polyxml generate --lang java --backend jackson --package com.enterprise.banking --out ./generated/java schemas/order.xsd

# Generate C++20 Modules with Glaze reflectionless serde
polyxml generate --lang cpp --mode modules --backend glaze --package enterprise::crm --out ./generated/cpp schemas/order.xsd

# Generate zero-copy Rust models with codecs and rkyv wire format
polyxml generate --lang rust --zero-copy --codecs --rkyv --out ./generated/rust schemas/order.xsd

# Generate TypeScript with tree-shakeable Valibot schemas
polyxml generate --lang ts --backend valibot --out ./generated/ts schemas/order.xsd

# Generate Go models with ByteDance Sonic JIT tags
polyxml generate --lang go --backend sonic --package crm --out ./generated/go schemas/order.xsd

# Generate C# record structs with Native AOT source generation
polyxml generate --lang csharp --record-kind struct --source-gen --namespace Enterprise.Crm --out ./generated/csharp schemas/order.xsd

# Multi-target compilation in a single invocation
polyxml generate \
  --lang python \
  --lang rust \
  --lang cpp \
  --lang java \
  --lang typescript \
  --lang go \
  --lang csharp \
  --out ./generated \
  schemas/pain.001.001.09.xsd

# Dry-run inspection without writing files to disk
polyxml generate --lang rust --dry-run schemas/order.xsd
```

### 2. `polyxml build`

Build an entire multi-target, multi-schema project declaratively from a `polyxml.toml` workspace manifest:

```bash
polyxml build --config polyxml.toml
```

### 3. `polyxml validate`

Statically check W3C XML schemas for structural validity, element types, and cycle topology:

```bash
polyxml validate schemas/*.xsd
```

---

## Workspace Manifest (`polyxml.toml`)

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
