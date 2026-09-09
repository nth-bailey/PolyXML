# polyxml

[![Crates.io](https://img.shields.io/crates/v/polyxml.svg)](https://crates.io/crates/polyxml)
[![Docs.rs](https://docs.rs/polyxml/badge.svg)](https://docs.rs/polyxml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/nth-bailey/PolyXML/blob/main/LICENSE)

**PolyXML** is a high-performance, zero-copy streaming XML data-binding core engine built in Rust. It serves as the foundational native engine powering language bindings across Python, C/C++, Java, and Node.js.

---

## Features

- **High Throughput & Low Latency**: Built on top of [`quick-xml`](https://crates.io/crates/quick-xml) and [`lexical-core`](https://crates.io/crates/lexical-core) for fast, zero-copy byte slice parsing.
- **Dynamic Schema-Driven**: Decouples data model definitions from parsing logic using `ModelSchema` and `FieldSchema`.
- **Bidirectional**: Fast streaming deserialization and serialization with optional indentation formatting.
- **Security Hardened**: Built-in recursion depth limits protect against XML entity expansion and deeply nested denial-of-service (Billion Laughs) attacks.
- **Zero Heavy Allocations**: Uses `smallvec` and slice lookups to minimize intermediate heap allocations.

---

## Installation

Add `polyxml` to your `Cargo.toml`:

```toml
[dependencies]
polyxml = "0.1.0"
```

Or run:

```bash
cargo add polyxml
```

---

## Quickstart

```rust
use std::sync::Arc;
use polyxml::{
    deserialize, serialize,
    FieldKind, FieldSchema, ModelSchema, PolyValue, ScalarType, ValueType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define your model schema
    let schema = ModelSchema::builder("User")
        .field(FieldSchema::new(
            "id",
            b"id",
            FieldKind::Attribute,
            ValueType::Scalar(ScalarType::Int),
        ))
        .field(FieldSchema::new(
            "name",
            b"name",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::String),
        ))
        .field(FieldSchema::new(
            "score",
            b"score",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Float),
        ))
        .build();

    let xml = br#"<User id="42"><name>Alice</name><score>98.6</score></User>"#;

    // 2. Deserialize XML bytes
    let user = deserialize(xml, Arc::clone(&schema))?;
    assert_eq!(user.get("id"), Some(&PolyValue::Int(42)));
    assert_eq!(user.get("name"), Some(&PolyValue::String("Alice".into())));
    assert_eq!(user.get("score"), Some(&PolyValue::Float(98.6)));

    // 3. Serialize back to formatted XML
    let output_xml = serialize("User", &user, &schema, Some(2))?;
    println!("{}", std::str::from_utf8(&output_xml)?);

    Ok(())
}
```

---

## Nested Models & Lists

```rust
use std::sync::Arc;
use polyxml::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};

// Sub-model
let item_schema = ModelSchema::builder("Item")
    .field(FieldSchema::new("sku", b"sku", FieldKind::Attribute, ValueType::Scalar(ScalarType::String)))
    .field(FieldSchema::new("qty", b"qty", FieldKind::Element, ValueType::Scalar(ScalarType::Int)))
    .build();

// Parent model containing repeated sub-models
let order_schema = ModelSchema::builder("Order")
    .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::String)))
    .field(FieldSchema::new(
        "items",
        b"item",
        FieldKind::Element,
        ValueType::List(Box::new(ValueType::Nested(item_schema))),
    ))
    .build();
```

---

## Security Limits

To defend against deeply nested payloads or XML entity expansion denial-of-service, specify a recursion depth limit:

```rust
let val = polyxml::deserialize_with_limit(xml_bytes, schema, 64)?;
```

---

## Documentation & Repository

- **Full Documentation**: [https://nth-bailey.github.io/PolyXML/](https://nth-bailey.github.io/PolyXML/)
- **API Reference**: [https://docs.rs/polyxml](https://docs.rs/polyxml)
- **Source Code**: [https://github.com/nth-bailey/PolyXML](https://github.com/nth-bailey/PolyXML)

---

## License

Licensed under the [MIT License](https://github.com/nth-bailey/PolyXML/blob/main/LICENSE).
