use std::sync::Arc;

use polyxml_core::error::PolyXmlError;
use polyxml_core::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml_core::value::PolyValue;
use polyxml_core::{deserialize, serialize};

#[test]
fn test_all_scalar_types() {
    let schema = ModelSchema::builder("Sensors")
        .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("sensor_name", b"name", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("temperature", b"temp", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .field(FieldSchema::new("is_calibrated", b"calibrated", FieldKind::Element, ValueType::Scalar(ScalarType::Bool)))
        .field(FieldSchema::new("timestamp", b"timestamp", FieldKind::Element, ValueType::Scalar(ScalarType::XmlDateTime)))
        .field(FieldSchema::new("reading_date", b"date", FieldKind::Element, ValueType::Scalar(ScalarType::XmlDate)))
        .field(FieldSchema::new("duration", b"duration", FieldKind::Element, ValueType::Scalar(ScalarType::XmlDuration)))
        .field(FieldSchema::new("precision_decimal", b"prec", FieldKind::Element, ValueType::Scalar(ScalarType::Decimal)))
        .build();

    let xml = br#"
    <Sensors id="999">
        <name>Thermostat Alpha</name>
        <temp>-12.345</temp>
        <calibrated>1</calibrated>
        <timestamp>2026-09-09T10:00:00Z</timestamp>
        <date>2026-09-09</date>
        <duration>PT1H30M</duration>
        <prec>123456789.987654321</prec>
    </Sensors>
    "#;

    let val = deserialize(xml, Arc::clone(&schema)).expect("Failed to deserialize scalars");
    assert_eq!(val.get("id"), Some(&PolyValue::Int(999)));
    assert_eq!(val.get("sensor_name"), Some(&PolyValue::String("Thermostat Alpha".into())));
    assert_eq!(val.get("temperature"), Some(&PolyValue::Float(-12.345)));
    assert_eq!(val.get("is_calibrated"), Some(&PolyValue::Bool(true)));
    assert_eq!(val.get("timestamp"), Some(&PolyValue::String("2026-09-09T10:00:00Z".into())));
    assert_eq!(val.get("reading_date"), Some(&PolyValue::String("2026-09-09".into())));
    assert_eq!(val.get("duration"), Some(&PolyValue::String("PT1H30M".into())));
    assert_eq!(val.get("precision_decimal"), Some(&PolyValue::String("123456789.987654321".into())));
}

#[test]
fn test_error_handling_invalid_scalars() {
    let schema = ModelSchema::builder("Device")
        .field(FieldSchema::new("port", b"port", FieldKind::Element, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("ratio", b"ratio", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
        .field(FieldSchema::new("online", b"online", FieldKind::Element, ValueType::Scalar(ScalarType::Bool)))
        .build();

    // Invalid integer
    let xml_bad_int = b"<Device><port>not-a-number</port></Device>";
    let err = deserialize(xml_bad_int, Arc::clone(&schema)).unwrap_err();
    match err {
        PolyXmlError::ScalarParseError { field, expected, value } => {
            assert_eq!(field, "port");
            assert_eq!(expected, "integer");
            assert_eq!(value, "not-a-number");
        }
        other => panic!("Unexpected error variant: {:?}", other),
    }

    // Invalid float
    let xml_bad_float = b"<Device><ratio>bad_float</ratio></Device>";
    let err = deserialize(xml_bad_float, Arc::clone(&schema)).unwrap_err();
    match err {
        PolyXmlError::ScalarParseError { field, expected, .. } => {
            assert_eq!(field, "ratio");
            assert_eq!(expected, "float");
        }
        other => panic!("Unexpected error variant: {:?}", other),
    }

    // Invalid boolean
    let xml_bad_bool = b"<Device><online>maybe</online></Device>";
    let err = deserialize(xml_bad_bool, Arc::clone(&schema)).unwrap_err();
    match err {
        PolyXmlError::ScalarParseError { field, expected, .. } => {
            assert_eq!(field, "online");
            assert!(expected.contains("boolean"));
        }
        other => panic!("Unexpected error variant: {:?}", other),
    }
}

#[test]
fn test_malformed_xml_syntax_error() {
    let schema = ModelSchema::builder("Root")
        .field(FieldSchema::new("val", b"val", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .build();

    let unclosed_xml = b"<Root><val>test";
    let err = deserialize(unclosed_xml, schema);
    assert!(err.is_err());
}

#[test]
fn test_xsi_nil_handling() {
    let schema = ModelSchema::builder("Payload")
        .field(FieldSchema::new("nullable_val", b"val", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("nullable_attr", b"attr", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .build();

    let xml = br#"<Payload><val xsi:nil="true"/></Payload>"#;
    let val = deserialize(xml, schema).expect("Deserialization failed for nil element");
    assert_eq!(val.get("nullable_val"), Some(&PolyValue::Null));
}

#[test]
fn test_roundtrip_serialization_with_indentation() {
    let child_schema = ModelSchema::builder("Child")
        .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
        .field(FieldSchema::new("name", b"name", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .build();

    let parent_schema = ModelSchema::builder("Parent")
        .field(FieldSchema::new("title", b"title", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
        .field(FieldSchema::new("children", b"child", FieldKind::Element, ValueType::List(Box::new(ValueType::Nested(Arc::clone(&child_schema))))))
        .build();

    let xml = br#"
    <Parent>
        <title>Family Tree</title>
        <child id="1"><name>Alice</name></child>
        <child id="2"><name>Bob</name></child>
    </Parent>
    "#;

    let val = deserialize(xml, Arc::clone(&parent_schema)).expect("Failed to deserialize parent");
    let serialized = serialize("Parent", &val, &parent_schema, Some(2)).expect("Failed to serialize with indent");
    let serialized_str = std::str::from_utf8(&serialized).unwrap();

    assert!(serialized_str.contains("  <title>Family Tree</title>"));
    assert!(serialized_str.contains(r#"<child id="1">"#));
    assert!(serialized_str.contains("<name>Alice</name>"));

    // Re-deserialize to verify data preservation
    let val_re = deserialize(&serialized, Arc::clone(&parent_schema)).expect("Failed to re-deserialize");
    assert_eq!(val, val_re);
}
