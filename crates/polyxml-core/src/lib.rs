pub mod converters;
pub mod error;
pub mod parser;
pub mod schema;
pub mod serializer;
pub mod value;

pub use error::{PolyXmlError, Result};
pub use parser::XmlDeserializer;
pub use schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
pub use serializer::XmlSerializer;
pub use value::PolyValue;

use std::sync::Arc;

/// High-level function to deserialize XML bytes into a dynamic PolyValue using a ModelSchema.
pub fn deserialize(xml_bytes: &[u8], schema: Arc<ModelSchema>) -> Result<PolyValue> {
    XmlDeserializer::deserialize(xml_bytes, schema)
}

/// High-level function to deserialize XML bytes with a custom maximum recursion depth limit.
pub fn deserialize_with_limit(
    xml_bytes: &[u8],
    schema: Arc<ModelSchema>,
    max_depth: usize,
) -> Result<PolyValue> {
    XmlDeserializer::deserialize_with_limit(xml_bytes, schema, max_depth)
}

/// High-level function to serialize a PolyValue into XML bytes using a ModelSchema.
pub fn serialize(
    root_name: &str,
    value: &PolyValue,
    schema: &ModelSchema,
    indent: Option<usize>,
) -> Result<Vec<u8>> {
    XmlSerializer::serialize(root_name, value, schema, indent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_serialization_deserialization() {
        // 1. Build schema
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
            .field(FieldSchema::new(
                "active",
                b"active",
                FieldKind::Element,
                ValueType::Scalar(ScalarType::Bool),
            ))
            .build();

        let xml = br#"<User id="101"><name>Ada Lovelace</name><score>99.5</score><active>true</active></User>"#;

        // 2. Deserialize
        let val = deserialize(xml, Arc::clone(&schema)).expect("deserialization failed");
        assert_eq!(val.get("id"), Some(&PolyValue::Int(101)));
        assert_eq!(
            val.get("name"),
            Some(&PolyValue::String("Ada Lovelace".into()))
        );
        assert_eq!(val.get("score"), Some(&PolyValue::Float(99.5)));
        assert_eq!(val.get("active"), Some(&PolyValue::Bool(true)));

        // 3. Serialize back
        let output_xml = serialize("User", &val, &schema, None).expect("serialization failed");
        let output_str = std::str::from_utf8(&output_xml).unwrap();

        // 4. Verify roundtrip output contains all tags
        assert!(output_str.contains(r#"id="101""#));
        assert!(output_str.contains("<name>Ada Lovelace</name>"));
        assert!(output_str.contains("<score>99.5</score>"));
        assert!(output_str.contains("<active>true</active>"));

        // 5. Re-deserialize from generated XML
        let val2 =
            deserialize(&output_xml, Arc::clone(&schema)).expect("re-deserialization failed");
        assert_eq!(val, val2);
    }

    #[test]
    fn test_nested_elements_and_lists() {
        let item_schema = ModelSchema::builder("Item")
            .field(FieldSchema::new(
                "title",
                b"title",
                FieldKind::Element,
                ValueType::Scalar(ScalarType::String),
            ))
            .field(FieldSchema::new(
                "qty",
                b"qty",
                FieldKind::Attribute,
                ValueType::Scalar(ScalarType::Int),
            ))
            .build();

        let order_schema = ModelSchema::builder("Order")
            .field(FieldSchema::new(
                "order_id",
                b"id",
                FieldKind::Attribute,
                ValueType::Scalar(ScalarType::String),
            ))
            .field(FieldSchema::new(
                "items",
                b"item",
                FieldKind::Element,
                ValueType::List(Box::new(ValueType::Nested(Arc::clone(&item_schema)))),
            ))
            .build();

        let xml = br#"
        <Order id="ORD-999">
            <item qty="2"><title>Widget A</title></item>
            <item qty="5"><title>Widget B</title></item>
        </Order>
        "#;

        let val =
            deserialize(xml, Arc::clone(&order_schema)).expect("nested deserialization failed");
        assert_eq!(
            val.get("order_id"),
            Some(&PolyValue::String("ORD-999".into()))
        );

        let items = val.get("items").unwrap().as_list().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].get("title"),
            Some(&PolyValue::String("Widget A".into()))
        );
        assert_eq!(items[0].get("qty"), Some(&PolyValue::Int(2)));
        assert_eq!(
            items[1].get("title"),
            Some(&PolyValue::String("Widget B".into()))
        );
        assert_eq!(items[1].get("qty"), Some(&PolyValue::Int(5)));
    }
}
