---
title: Go Guide (Cgo)
description: High-throughput XML data binding in Go using PolyXML.
---

# Go Guide

PolyXML's Go package replaces `encoding/xml` with a high-throughput Cgo bridge.

## Installation

```bash
go get github.com/nth-bailey/PolyXML/bindings/go
```

## Example Usage

```go
package main

import (
    "fmt"
    "github.com/nth-bailey/PolyXML/bindings/go"
)

func main() {
    builder, _ := polyxml.NewSchemaBuilder("Node")
    builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
    builder.AddField("ip", "ip", polyxml.FieldElement, polyxml.ScalarString)
    schema, _ := builder.Build()

    xml := []byte(`<Node id="1"><ip>192.168.1.100</ip></Node>`)

    // Deserialize
    val, err := polyxml.Deserialize(xml, schema)
    if err != nil {
        panic(err)
    }

    ip, _ := val.GetField("ip").GetString()
    fmt.Println("Node IP:", ip)

    // Serialize
    out, _ := polyxml.Serialize("Node", val, schema, 2)
    fmt.Println(string(out))
}
```
