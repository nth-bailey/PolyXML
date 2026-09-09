---
title: Rust Core Guide
description: Using PolyXML's native Rust engine directly in Rust applications for high-throughput, zero-allocation XML processing.
---

# Rust Native Core Guide

The `polyxml` crate is the pure, idiomatic Rust core engine driving the entire PolyXML ecosystem. It provides ultra-fast streaming serialization and deserialization with zero FFI overhead, zero C dependencies, and no intermediate DOM allocations.

---

## 📦 Cargo Installation

Add `polyxml` to your `Cargo.toml`:

```toml
[dependencies]
polyxml = "0.1"
```

---

## 1. Basic Models: Attributes vs. Elements

PolyXML distinguishes between XML attributes, child elements, and text content through `FieldKind`.

```rust
use std::sync::Arc;
use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::{deserialize, serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define the schema programmatically
    let schema = ModelSchema::builder("Telemetry")
        .field(FieldSchema::new("device_id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)).required())
        .field(FieldSchema::new("altitude", b"altitude", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .field(FieldSchema::new("armed", b"armed", FieldKind::Element, ValueType::Scalar(ScalarType::Bool)))
        .field(FieldSchema::new("callsign", b"callsign", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .build();

    let xml = br#"<Telemetry id="402">
        <altitude>31500.75</altitude>
        <armed>true</armed>
        <callsign>GHOST-1</callsign>
    </Telemetry>"#;

    // 2. Deserialize XML into a PolyValue map
    let val = deserialize(xml, Arc::clone(&schema))?;
    
    // 3. Extract strongly-typed scalar values
    let device_id = val.get("device_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let altitude = val.get("altitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let armed = val.get("armed").and_then(|v| v.as_bool()).unwrap_or(false);
    let callsign = val.get("callsign").and_then(|v| v.as_str()).unwrap_or("");

    println!("Telemetry: Device {device_id} ({callsign}) at {altitude}ft, Armed: {armed}");

    // 4. Serialize back to formatted XML bytes
    let output = serialize("Telemetry", &val, &schema, Some(2))?;
    println!("Serialized XML:\n{}", std::str::from_utf8(&output)?);

    Ok(())
}
```

---

## 2. Nested Structures & Repeated Collections (`List`)

PolyXML natively supports complex hierarchical data models, including nested sub-objects (`ValueType::Nested`) and repeated collections (`ValueType::List`).

```rust
use std::sync::Arc;
use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::{deserialize, serialize, PolyValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the child schema (Item)
    let item_schema = ModelSchema::builder("Item")
        .field(FieldSchema::new("sku", b"sku", FieldKind::Attribute, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("qty", b"qty", FieldKind::Element, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("price", b"price", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .build();

    // Define the parent schema (Order) containing a list of Items
    let order_schema = ModelSchema::builder("Order")
        .field(FieldSchema::new("order_id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new(
            "items",
            b"Item",
            FieldKind::Element,
            ValueType::List(Box::new(ValueType::Nested(Arc::clone(&item_schema)))),
        ))
        .build();

    let xml = br#"
    <Order id="9801">
        <Item sku="PART-A"><qty>5</qty><price>19.99</price></Item>
        <Item sku="PART-B"><qty>2</qty><price>45.50</price></Item>
    </Order>
    "#;

    let order = deserialize(xml, Arc::clone(&order_schema))?;
    
    // Inspect repeated elements
    if let Some(PolyValue::List(items)) = order.get("items") {
        for item in items {
            let sku = item.get("sku").and_then(|v| v.as_str()).unwrap_or("");
            let qty = item.get("qty").and_then(|v| v.as_i64()).unwrap_or(0);
            let price = item.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            println!("- Item: SKU={sku}, Qty={qty}, Total=${:.2}", (qty as f64) * price);
        }
    }

    Ok(())
}
```

---

## 3. Constant-Memory Streaming with `XmlItemStream`

When parsing multi-gigabyte XML feeds (e.g. PubMed, Wikipedia, SEC EDGAR, ISO 20022), loading the full document into memory is prohibitive. `XmlItemStream` provides a streaming iterator that yields individual record subtrees with **$O(1)$ constant memory**.

```rust
use std::sync::Arc;
use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::XmlItemStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let item_schema = ModelSchema::builder("Product")
        .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("title", b"title", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("price", b"price", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .build();

    let large_xml_stream = br#"
    <Catalog>
        <Product id="1"><title>Sensor Module A</title><price>12.50</price></Product>
        <Product id="2"><title>Actuator Hub B</title><price>89.00</price></Product>
        <Product id="3"><title>Telemetry Unit C</title><price>245.99</price></Product>
    </Catalog>
    "#;

    // Stream records matching tag "Product" without materializing the <Catalog> DOM
    let mut stream = XmlItemStream::new(&large_xml_stream[..], b"Product", Arc::clone(&item_schema));

    let mut count = 0;
    while let Some(result) = stream.next() {
        let product = result?;
        let title = product.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let price = product.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        println!("Streamed product #{}: {} (${:.2})", count + 1, title, price);
        count += 1;
    }

    println!("Total products processed: {count}");
    Ok(())
}
```

---

## 4. Nullability, Empty Elements, and `xsi:nil`

In production XML feeds, elements may be empty, omitted, or explicitly marked null via XML Schema Instance attributes (`xsi:nil="true"`). PolyXML natively handles all three scenarios:

```rust
use std::sync::Arc;
use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::{deserialize, PolyValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = ModelSchema::builder("Record")
        .field(FieldSchema::new("id", b"id", FieldKind::Element, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("notes", b"notes", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("status", b"status", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .build();

    let xml = br#"
    <Record xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
        <id>100</id>
        <notes xsi:nil="true"/>
        <status></status>
    </Record>
    "#;

    let record = deserialize(xml, schema)?;

    // notes is parsed as PolyValue::Null due to xsi:nil="true"
    assert_eq!(record.get("notes"), Some(&PolyValue::Null));

    // empty <status></status> parses as empty string
    assert_eq!(record.get("status").and_then(|v| v.as_str()), Some(""));

    Ok(())
}
```

---

## 5. CDATA Sections & XML Entity Escaping

PolyXML automatically unescapes standard XML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`) as well as numeric character references (`&#65;`, `&#x41;`), and extracts raw text within `<![CDATA[...]]>` blocks:

```rust
use std::sync::Arc;
use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::deserialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = ModelSchema::builder("Payload")
        .field(FieldSchema::new("script", b"script", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("desc", b"desc", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .build();

    let xml = br#"
    <Payload>
        <script><![CDATA[if (a < 10 && b > 20) { return true; }]]></script>
        <desc>Tom &amp; Jerry &quot;Classic&quot;</desc>
    </Payload>
    "#;

    let val = deserialize(xml, schema)?;
    assert_eq!(val.get("script").unwrap().as_str().unwrap(), "if (a < 10 && b > 20) { return true; }");
    assert_eq!(val.get("desc").unwrap().as_str().unwrap(), "Tom & Jerry \"Classic\"");

    Ok(())
}
```

---

## 6. High-Throughput Production Best Practices

To achieve the maximum throughput from `polyxml-core`:

1. **Reuse Schema Instances**: Construct `Arc<ModelSchema>` once at startup or store it in a static `LazyLock`. Passing `Arc::clone(&schema)` is an atomic reference bump that avoids re-indexing fields.
2. **Pre-allocate Buffers**: For serialization loops, pass an existing `Vec<u8>` or reuse serialization buffers across requests to avoid heap allocations.
3. **Use `XmlItemStream` for Files > 5 MB**: For large XML feeds, streaming ensures your process memory remains constant regardless of file size.
4. **Thread Safety**: Both `ModelSchema` and `PolyValue` are fully `Send + Sync`, making them ideal for parallel processing with `rayon` or multi-threaded Tokio runtimes.
