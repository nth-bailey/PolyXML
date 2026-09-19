# PolyXML CLI (`polyxml`)

Unified developer command-line interface and schema compiler toolchain for PolyXML.

## Usage

```bash
# Compile schema to target models
polyxml generate --lang python --out ./generated schemas/order.xsd

# Multi-target compilation in a single invocation
polyxml generate --lang python --lang rust --lang cpp --out ./generated schemas/pain.001.001.09.xsd

# Build project using workspace manifest (polyxml.toml)
polyxml build --config polyxml.toml

# Validate schema structure
polyxml validate schemas/*.xsd
```

## `polyxml.toml` Manifest

```toml
[workspace]
name = "enterprise-schemas"
schemas = ["schemas/iso20022/*.xsd"]
include_dirs = ["schemas/common/"]
output_base_dir = "./generated"

[[generate]]
target = "python"
output = "src/generated/python"
backend = "pydantic-v2"

[[generate]]
target = "rust"
output = "src/generated/rust"
zero_copy = true

[[generate]]
target = "java"
output = "src/generated/java"
package = "com.enterprise.banking.iso20022"
```
