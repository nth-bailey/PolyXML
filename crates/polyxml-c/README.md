<p align="center">
  <a href="https://github.com/polyxml/PolyXML">
    <img src="https://raw.githubusercontent.com/polyxml/PolyXML/main/docs/assets/brand/logo_polyxml_banner.png" alt="PolyXML" width="800">
  </a>
</p>

# polyxml-c

<p align="center">
  <a href="https://crates.io/crates/polyxml-c"><img src="https://img.shields.io/crates/v/polyxml-c.svg?logo=rust&label=crates.io" alt="crates.io"></a>
  <a href="https://en.wikipedia.org/wiki/C99"><img src="https://img.shields.io/badge/C-C99%20%7C%20C11-00599C.svg?logo=c" alt="C: C99"></a>
  <a href="https://polyxml.github.io/PolyXML/"><img src="https://img.shields.io/badge/docs-zensical-blue.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>


**polyxml-c** provides the standard C-ABI shared and static library (`libpolyxml`) and C headers (`polyxml.h`) for **PolyXML**.

It exposes high-performance XML streaming and schema construction to C, C++, and any foreign function interface (FFI) runtime.

---

## Features

- **Pure C99 API**: Clean, thread-safe C header (`polyxml.h`).
- **Opaque Pointers**: Safe memory management with explicit `polyxml_schema_free`, `polyxml_value_free`, and `polyxml_buffer_free`.
- **Zero Heavy Allocations**: Exposes PolyXML's native Rust streaming engine to native codebases.

---

## Building

```bash
cargo build --release -p polyxml-c
```

This generates:
- Linux / macOS: `target/release/libpolyxml.so` or `libpolyxml.dylib`
- Windows: `target/release/polyxml.dll` and `polyxml.dll.lib`

Headers are located in `crates/polyxml-c/include/polyxml.h`.

---

## Example (C99)

```c
#include <stdio.h>
#include <polyxml.h>

int main(void) {
    printf("PolyXML C-API Version: %s\n", polyxml_version());

    // 1. Build a schema
    PolyXmlSchemaBuilder* b = polyxml_builder_new("User");
    polyxml_builder_add_field(b, "id", "id", POLYXML_FIELD_ATTRIBUTE, POLYXML_TYPE_INT);
    polyxml_builder_add_field(b, "name", "name", POLYXML_FIELD_ELEMENT, POLYXML_TYPE_STRING);
    PolyXmlModelSchema* schema = polyxml_builder_build(b);

    // 2. Deserialize XML
    const char* xml = "<User id=\"101\"><name>Ada</name></User>";
    PolyXmlValue* val = polyxml_deserialize(xml, strlen(xml), schema);

    // 3. Access values
    int64_t id = polyxml_value_get_int(val, "id");
    const char* name = polyxml_value_get_string(val, "name");
    printf("User %lld: %s\n", (long long)id, name);

    // 4. Cleanup
    polyxml_value_free(val);
    polyxml_schema_free(schema);
    return 0;
}
```

---

## License

Licensed under the [MIT License](https://github.com/polyxml/PolyXML/blob/main/LICENSE).
