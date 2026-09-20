use std::collections::HashMap;
use std::sync::Arc;

use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::value::PolyValue;
use polyxml::{deserialize_json, serialize_json};

#[test]
fn test_json_primitives_and_alias_modes() {
    let schema = ModelSchema::builder("Customer")
        .field(FieldSchema::new(
            "customer_id",
            b"customerId",
            FieldKind::Attribute,
            ValueType::Scalar(ScalarType::Int),
        ))
        .field(FieldSchema::new(
            "company_name",
            b"companyName",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::String),
        ))
        .field(FieldSchema::new(
            "balance",
            b"balance",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Float),
        ))
        .field(FieldSchema::new(
            "active",
            b"isActive",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Bool),
        ))
        .build();

    let mut map = HashMap::new();
    map.insert("customer_id".to_string(), PolyValue::Int(1001));
    map.insert(
        "company_name".to_string(),
        PolyValue::String("Acme Corp".to_string()),
    );
    map.insert("balance".to_string(), PolyValue::Float(1234.56));
    map.insert("active".to_string(), PolyValue::Bool(true));
    let poly_val = PolyValue::Object(map);

    // 1. By alias = true (default schema names)
    let alias_bytes = serialize_json(&poly_val, &schema, None, true).expect("serialize alias");
    let alias_str = std::str::from_utf8(&alias_bytes).unwrap();
    assert!(alias_str.contains("\"customerId\":1001"));
    assert!(alias_str.contains("\"companyName\":\"Acme Corp\""));
    assert!(alias_str.contains("\"isActive\":true"));

    // 2. By alias = false (internal field names)
    let direct_bytes = serialize_json(&poly_val, &schema, None, false).expect("serialize direct");
    let direct_str = std::str::from_utf8(&direct_bytes).unwrap();
    assert!(direct_str.contains("\"customer_id\":1001"));
    assert!(direct_str.contains("\"company_name\":\"Acme Corp\""));
    assert!(direct_str.contains("\"active\":true"));

    // 3. Roundtrip deserialization with alias
    let parsed_alias = deserialize_json(&alias_bytes, Arc::clone(&schema)).expect("deserialize");
    assert_eq!(parsed_alias.get("customer_id"), Some(&PolyValue::Int(1001)));
    assert_eq!(
        parsed_alias.get("company_name"),
        Some(&PolyValue::String("Acme Corp".to_string()))
    );
    assert_eq!(parsed_alias.get("active"), Some(&PolyValue::Bool(true)));

    // 4. Roundtrip deserialization with internal snake_case names
    let parsed_direct = deserialize_json(&direct_bytes, Arc::clone(&schema)).expect("deserialize");
    assert_eq!(
        parsed_direct.get("customer_id"),
        Some(&PolyValue::Int(1001))
    );
    assert_eq!(
        parsed_direct.get("company_name"),
        Some(&PolyValue::String("Acme Corp".to_string()))
    );
    assert_eq!(parsed_direct.get("active"), Some(&PolyValue::Bool(true)));
}

#[test]
fn test_json_nested_objects_and_lists() {
    let item_schema = Arc::new(
        ModelSchema::builder("Item")
            .field(FieldSchema::new(
                "sku",
                b"skuCode",
                FieldKind::Attribute,
                ValueType::Scalar(ScalarType::String),
            ))
            .field(FieldSchema::new(
                "price",
                b"unitPrice",
                FieldKind::Element,
                ValueType::Scalar(ScalarType::Float),
            ))
            .build(),
    );

    let order_schema = ModelSchema::builder("Order")
        .field(FieldSchema::new(
            "order_id",
            b"orderId",
            FieldKind::Attribute,
            ValueType::Scalar(ScalarType::Int),
        ))
        .field(FieldSchema::new(
            "items",
            b"itemsList",
            FieldKind::Element,
            ValueType::List(Box::new(ValueType::Nested(Arc::clone(&item_schema)))),
        ))
        .build();

    let mut item1 = HashMap::new();
    item1.insert("sku".to_string(), PolyValue::String("ITEM-1".to_string()));
    item1.insert("price".to_string(), PolyValue::Float(19.99));

    let mut item2 = HashMap::new();
    item2.insert("sku".to_string(), PolyValue::String("ITEM-2".to_string()));
    item2.insert("price".to_string(), PolyValue::Float(49.99));

    let mut order_map = HashMap::new();
    order_map.insert("order_id".to_string(), PolyValue::Int(555));
    order_map.insert(
        "items".to_string(),
        PolyValue::List(vec![PolyValue::Object(item1), PolyValue::Object(item2)]),
    );
    let order_val = PolyValue::Object(order_map);

    let json_bytes = serialize_json(&order_val, &order_schema, Some(2), true).unwrap();
    let json_str = std::str::from_utf8(&json_bytes).unwrap();
    assert!(json_str.contains("\"orderId\": 555"));
    assert!(json_str.contains("\"itemsList\": ["));
    assert!(json_str.contains("\"skuCode\": \"ITEM-1\""));
    assert!(json_str.contains("\"unitPrice\": 19.99"));

    let parsed = deserialize_json(&json_bytes, Arc::clone(&order_schema)).unwrap();
    assert_eq!(parsed.get("order_id"), Some(&PolyValue::Int(555)));
    if let Some(PolyValue::List(items)) = parsed.get("items") {
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].get("sku"),
            Some(&PolyValue::String("ITEM-1".to_string()))
        );
        assert_eq!(items[0].get("price"), Some(&PolyValue::Float(19.99)));
        assert_eq!(
            items[1].get("sku"),
            Some(&PolyValue::String("ITEM-2".to_string()))
        );
        assert_eq!(items[1].get("price"), Some(&PolyValue::Float(49.99)));
    } else {
        panic!("Expected items list");
    }
}

#[test]
fn test_json_scalar_coercion() {
    let schema = ModelSchema::builder("Coerce")
        .field(FieldSchema::new(
            "int_val",
            b"intVal",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Int),
        ))
        .field(FieldSchema::new(
            "float_val",
            b"floatVal",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Float),
        ))
        .field(FieldSchema::new(
            "bool_val",
            b"boolVal",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Bool),
        ))
        .field(FieldSchema::new(
            "dec_val",
            b"decVal",
            FieldKind::Element,
            ValueType::Scalar(ScalarType::Decimal),
        ))
        .build();

    // Pass string representations for numbers and booleans
    let json = br#"{
        "intVal": "42",
        "floatVal": "12.75",
        "boolVal": "true",
        "decVal": 999.88
    }"#;

    let parsed = deserialize_json(json, schema).unwrap();
    assert_eq!(parsed.get("int_val"), Some(&PolyValue::Int(42)));
    assert_eq!(parsed.get("float_val"), Some(&PolyValue::Float(12.75)));
    assert_eq!(parsed.get("bool_val"), Some(&PolyValue::Bool(true)));
    assert_eq!(
        parsed.get("dec_val"),
        Some(&PolyValue::String("999.88".to_string()))
    );
}

#[test]
fn test_polyvalue_direct_json_methods() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), PolyValue::String("Ada".to_string()));
    map.insert("count".to_string(), PolyValue::Int(3));
    let val = PolyValue::Object(map);

    let bytes = val.to_json(Some(2)).unwrap();
    let s = std::str::from_utf8(&bytes).unwrap();
    assert!(s.contains("\"name\": \"Ada\""));

    let back = PolyValue::from_json(&bytes).unwrap();
    assert_eq!(
        back.get("name"),
        Some(&PolyValue::String("Ada".to_string()))
    );
    assert_eq!(back.get("count"), Some(&PolyValue::Int(3)));
}
