<p align="center">
  <a href="https://github.com/polyxml/PolyXML">
    <img src="https://raw.githubusercontent.com/polyxml/PolyXML/main/docs/assets/brand/logo_polyxml_banner.png" alt="PolyXML" width="800">
  </a>
</p>

# PolyXML Go Bindings

<p align="center">
  <a href="https://pkg.go.dev/github.com/polyxml/PolyXML/bindings/go"><img src="https://pkg.go.dev/badge/github.com/polyxml/PolyXML/bindings/go.svg" alt="Go Reference"></a>
  <a href="https://go.dev"><img src="https://img.shields.io/badge/Go-1.22%2B-00ADD8.svg?logo=go&logoColor=white" alt="Go: 1.22+"></a>
  <a href="https://polyxml.github.io/PolyXML/languages/go/"><img src="https://img.shields.io/badge/docs-zensical-blue.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

High-performance native XML data-binding and streaming runtime for Go.

Powered by `polyxml-core` written in Rust and native Cgo bindings, providing direct C-speed XML serialization and deserialization.

---

## Installation

```bash
go get github.com/polyxml/PolyXML/bindings/go
```

---

## Features

- **⚡ High-Throughput Streaming**: Zero DOM allocations, powered by the Rust `polyxml-core` engine.
- **🛡️ Type-Safe Schema Builder**: Programmatic schema definition with validation.
- **🔄 Bidirectional Serialization**: Fast streaming XML deserialization into generic `Value` trees and XML emission.
- **🧩 Direct Code Generation**: Compatible with structs generated via `polyxml generate --lang go`.

---

## Quickstart

```go
package main

import (
	"fmt"
	"log"

	"github.com/polyxml/PolyXML/bindings/go"
)

func main() {
	// 1. Build a model schema
	b, err := polyxml.NewSchemaBuilder("User")
	if err != nil {
		log.Fatal(err)
	}
	b.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	b.AddField("name", "name", polyxml.FieldElement, polyxml.ScalarString)
	schema := b.Build()
	defer schema.Free()

	// 2. Deserialize XML
	xml := `<User id="101"><name>Ada Lovelace</name></User>`
	val, err := polyxml.Deserialize(xml, schema)
	if err != nil {
		log.Fatal(err)
	}
	defer val.Free()

	// 3. Extract strongly-typed fields
	id, _ := val.GetInt("id")
	name, _ := val.GetString("name")
	fmt.Printf("User %d: %s\n", id, name)
}
```

---

## License

MIT
