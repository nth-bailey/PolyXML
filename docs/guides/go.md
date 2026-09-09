---
title: Go Guide (Cgo)
description: High-throughput XML data binding in Go using PolyXML's native Cgo bridge to replace slow encoding/xml.
---

# Go Guide (Cgo)

PolyXML provides high-throughput native XML parsing for Go, replacing the standard library's reflection-heavy `encoding/xml` with a zero-copy Rust streaming engine connected via Cgo.

---

## 📦 Installation

```bash
go get github.com/nth-bailey/PolyXML/bindings/go
```

*Requirements: Cgo enabled (`CGO_ENABLED=1`) and `libpolyxml` installed or present in library search path.*

---

## 1. Schema Definition & Basic Parsing

Use `NewSchemaBuilder` to declare attributes, child elements, and scalar types:

```go
package main

import (
	"fmt"
	"log"

	"github.com/nth-bailey/PolyXML/bindings/go"
)

func main() {
	// 1. Build schema
	builder, err := polyxml.NewSchemaBuilder("ServerNode")
	if err != nil {
		log.Fatalf("Failed to create builder: %v", err)
	}

	builder.AddField("node_id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	builder.AddField("hostname", "hostname", polyxml.FieldElement, polyxml.ScalarString)
	builder.AddField("ip_address", "ip", polyxml.FieldElement, polyxml.ScalarString)
	builder.AddField("is_active", "active", polyxml.FieldElement, polyxml.ScalarBool)
	builder.AddField("load_average", "load", polyxml.FieldElement, polyxml.ScalarFloat)

	schema, err := builder.Build()
	if err != nil {
		log.Fatalf("Failed to build schema: %v", err)
	}

	xmlPayload := []byte(`
		<ServerNode id="42">
			<hostname>worker-us-east-1</hostname>
			<ip>10.0.1.42</ip>
			<active>true</active>
			<load>0.68</load>
		</ServerNode>
	`)

	// 2. Deserialize XML bytes
	val, err := polyxml.Deserialize(xmlPayload, schema)
	if err != nil {
		log.Fatalf("Deserialization error: %v", err)
	}

	// 3. Extract typed fields
	id, _ := val.GetField("node_id").GetInt()
	host, _ := val.GetField("hostname").GetString()
	ip, _ := val.GetField("ip_address").GetString()
	active, _ := val.GetField("is_active").GetBool()
	load, _ := val.GetField("load_average").GetFloat()

	fmt.Printf("Node #%d [%s (%s)]: Active=%v, Load=%.2f\n", id, host, ip, active, load)
}
```

---

## 2. Formatting & Serialization

Serialize native values into formatted XML byte slices:

```go
package main

import (
	"fmt"
	"log"

	"github.com/nth-bailey/PolyXML/bindings/go"
)

func main() {
	builder, _ := polyxml.NewSchemaBuilder("Message")
	builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	builder.AddField("text", "text", polyxml.FieldElement, polyxml.ScalarString)
	schema, _ := builder.Build()

	xml := []byte(`<Message id="77"><text>Cluster sync complete</text></Message>`)
	val, err := polyxml.Deserialize(xml, schema)
	if err != nil {
		log.Fatal(err)
	}

	// Serialize with 2-space indentation
	out, err := polyxml.Serialize("Message", val, schema, 2)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Formatted XML output:\n" + string(out))
}
```

---

## 3. Idiomatic Go Struct Adapter Pattern

In production Go microservices, map `polyxml.Value` into domain structs for clean separation of concerns:

```go
package main

import (
	"fmt"
	"github.com/nth-bailey/PolyXML/bindings/go"
)

type TradeConfirmation struct {
	TradeID     int64
	Symbol      string
	Price       float64
	Quantity    int64
	Settled     bool
}

func parseTrade(xml []byte, schema *polyxml.Schema) (*TradeConfirmation, error) {
	val, err := polyxml.Deserialize(xml, schema)
	if err != nil {
		return nil, err
	}

	tradeID, _ := val.GetField("trade_id").GetInt()
	symbol, _ := val.GetField("symbol").GetString()
	price, _ := val.GetField("price").GetFloat()
	qty, _ := val.GetField("quantity").GetInt()
	settled, _ := val.GetField("settled").GetBool()

	return &TradeConfirmation{
		TradeID:  tradeID,
		Symbol:   symbol,
		Price:    price,
		Quantity: qty,
		Settled:  settled,
	}, nil
}

func main() {
	builder, _ := polyxml.NewSchemaBuilder("Trade")
	builder.AddField("trade_id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	builder.AddField("symbol", "symbol", polyxml.FieldElement, polyxml.ScalarString)
	builder.AddField("price", "price", polyxml.FieldElement, polyxml.ScalarFloat)
	builder.AddField("quantity", "quantity", polyxml.FieldElement, polyxml.ScalarInt)
	builder.AddField("settled", "settled", polyxml.FieldElement, polyxml.ScalarBool)
	schema, _ := builder.Build()

	xml := []byte(`<Trade id="9821"><symbol>AAPL</symbol><price>224.50</price><quantity>500</quantity><settled>true</settled></Trade>`)
	trade, err := parseTrade(xml, schema)
	if err != nil {
		panic(err)
	}

	fmt.Printf("Executed Trade: %d shares of %s at $%.2f (ID: %d)\n", 
		trade.Quantity, trade.Symbol, trade.Price, trade.TradeID)
}
```

---

## 4. HTTP Service Integration (Replacing `encoding/xml`)

Use PolyXML in high-throughput HTTP handlers to eliminate garbage collection pressure caused by reflection in standard `encoding/xml`:

```go
package main

import (
	"io"
	"log"
	"net/http"
	"sync"

	"github.com/nth-bailey/PolyXML/bindings/go"
)

var (
	payloadSchema *polyxml.Schema
	schemaOnce    sync.Once
)

func getSchema() *polyxml.Schema {
	schemaOnce.Do(func() {
		b, err := polyxml.NewSchemaBuilder("Event")
		if err != nil {
			log.Fatalf("Failed to init schema: %v", err)
		}
		b.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
		b.AddField("type", "type", polyxml.FieldElement, polyxml.ScalarString)
		b.AddField("timestamp", "timestamp", polyxml.FieldElement, polyxml.ScalarString)
		payloadSchema, _ = b.Build()
	})
	return payloadSchema
}

func xmlHandler(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}

	schema := getSchema()
	val, err := polyxml.Deserialize(body, schema)
	if err != nil {
		http.Error(w, "Invalid XML payload", http.StatusUnprocessableEntity)
		return
	}

	eventType, _ := val.GetField("type").GetString()
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(fmt.Sprintf("Event '%s' processed successfully", eventType)))
}
```

---

## 5. Memory Management & Garbage Collection

PolyXML Go bindings bind Go's GC to native Rust resources through `runtime.SetFinalizer`:
- When a `*polyxml.Schema` or `*polyxml.Value` object becomes unreachable in Go, the Go garbage collector invokes the finalizer which calls `C.polyxml_schema_free` or `C.polyxml_value_free`.
- **Best Practice**: In high-throughput batch loops, avoid creating new `Schema` objects repeatedly. Construct global or long-lived schemas to eliminate allocation cycles.
