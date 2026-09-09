---
title: Rust Core Guide
description: Using PolyXML's native Rust engine directly in Rust applications.
---

# Rust Core Guide

The `polyxml-core` crate is a 100% pure Rust library with zero FFI overhead.

## Cargo Configuration

Add `polyxml-core` to your `Cargo.toml`:

```toml
[dependencies]
polyxml-core = "0.1"
```

## Creating a Schema Programmatically

```rust
use std::sync::Arc;
use polyxml_core::schema::{ModelSchema, FieldSchema, FieldKind, ScalarType, ValueType};
use polyxml_core::{deserialize, serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = ModelSchema::builder("Telemetry")
        .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("altitude", b"altitude", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .field(FieldSchema::new("armed", b"armed", FieldKind::Element, ValueType::Scalar(ScalarType::Bool)))
        .build();

    let xml = br#"<Telemetry id="77"><altitude>12500.5</altitude><armed>true</armed></Telemetry>"#;

    // Deserialization
    let val = deserialize(xml, Arc::clone(&schema))?;
    println!("Altitude: {}", val.get("altitude").unwrap().as_f64().unwrap());

    // Serialization
    let output = serialize("Telemetry", &val, &schema, Some(2))?;
    println!("{}", std::str::from_utf8(&output)?);

    Ok(())
}
```
