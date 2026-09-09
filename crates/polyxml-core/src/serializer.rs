use std::io::Cursor;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use crate::error::{PolyXmlError, Result};
use crate::schema::{FieldKind, ModelSchema, ValueType};
use crate::value::PolyValue;

pub struct XmlSerializer;

impl XmlSerializer {
    pub fn serialize(
        root_name: &str,
        value: &PolyValue,
        schema: &ModelSchema,
        indent: Option<usize>,
    ) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::with_capacity(512));
        let mut writer = match indent {
            Some(spaces) => Writer::new_with_indent(&mut buffer, b' ', spaces),
            None => Writer::new(&mut buffer),
        };

        Self::write_model(&mut writer, root_name.as_bytes(), value, schema)?;

        Ok(buffer.into_inner())
    }

    fn format_scalar_to<'a>(
        val: &'a PolyValue,
        buf: &'a mut [u8; lexical_core::BUFFER_SIZE],
    ) -> Option<&'a str> {
        match val {
            PolyValue::String(s) => Some(s.as_str()),
            PolyValue::Int(i) => {
                let bytes = lexical_core::write(*i, buf);
                std::str::from_utf8(bytes).ok()
            }
            PolyValue::Float(f) => {
                let bytes = lexical_core::write(*f, buf);
                std::str::from_utf8(bytes).ok()
            }
            PolyValue::Bool(b) => Some(if *b { "true" } else { "false" }),
            _ => None,
        }
    }

    fn write_model<W: std::io::Write>(
        writer: &mut Writer<W>,
        tag_name: &[u8],
        value: &PolyValue,
        schema: &ModelSchema,
    ) -> Result<()> {
        let obj = match value {
            PolyValue::Object(o) => o,
            _ => {
                return Err(PolyXmlError::SerializationError(
                    "Expected Object value for model".into(),
                ))
            }
        };

        let mut elem = BytesStart::new(std::str::from_utf8(tag_name)?);

        // 1. Collect and write attributes
        for field in &schema.fields {
            if field.kind == FieldKind::Attribute {
                if let Some(val) = obj.get(&field.name) {
                    if !val.is_null() {
                        let attr_name = std::str::from_utf8(&field.xml_name)?;
                        let mut buf = [0u8; lexical_core::BUFFER_SIZE];
                        if let Some(attr_str) = Self::format_scalar_to(val, &mut buf) {
                            elem.push_attribute((attr_name, attr_str));
                        }
                    }
                }
            }
        }

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;

        // 2. Write text content if present
        if let Some(text_idx) = schema.text_field {
            let field = &schema.fields[text_idx];
            if let Some(val) = obj.get(&field.name) {
                let mut buf = [0u8; lexical_core::BUFFER_SIZE];
                if let Some(text_content) = Self::format_scalar_to(val, &mut buf) {
                    if !text_content.is_empty() {
                        writer
                            .write_event(Event::Text(BytesText::new(text_content)))
                            .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;
                    }
                }
            }
        }

        // 3. Write child elements
        for field in &schema.fields {
            if field.kind == FieldKind::Element {
                if let Some(val) = obj.get(&field.name) {
                    if val.is_null() {
                        continue;
                    }
                    match &field.val_type {
                        ValueType::Scalar(_) => {
                            let mut buf = [0u8; lexical_core::BUFFER_SIZE];
                            if let Some(text) = Self::format_scalar_to(val, &mut buf) {
                                let child_tag = std::str::from_utf8(&field.xml_name)?;
                                writer
                                    .write_event(Event::Start(BytesStart::new(child_tag)))
                                    .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;
                                writer
                                    .write_event(Event::Text(BytesText::new(text)))
                                    .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;
                                writer
                                    .write_event(Event::End(BytesEnd::new(child_tag)))
                                    .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;
                            }
                        }
                        ValueType::List(inner) => {
                            if let PolyValue::List(items) = val {
                                for item in items {
                                    match inner.as_ref() {
                                        ValueType::Scalar(_) => {
                                            let mut buf = [0u8; lexical_core::BUFFER_SIZE];
                                            if let Some(text) =
                                                Self::format_scalar_to(item, &mut buf)
                                            {
                                                let child_tag =
                                                    std::str::from_utf8(&field.xml_name)?;
                                                writer
                                                    .write_event(Event::Start(BytesStart::new(
                                                        child_tag,
                                                    )))
                                                    .map_err(|e| {
                                                        PolyXmlError::SerializationError(
                                                            e.to_string(),
                                                        )
                                                    })?;
                                                writer
                                                    .write_event(Event::Text(BytesText::new(text)))
                                                    .map_err(|e| {
                                                        PolyXmlError::SerializationError(
                                                            e.to_string(),
                                                        )
                                                    })?;
                                                writer
                                                    .write_event(Event::End(BytesEnd::new(
                                                        child_tag,
                                                    )))
                                                    .map_err(|e| {
                                                        PolyXmlError::SerializationError(
                                                            e.to_string(),
                                                        )
                                                    })?;
                                            }
                                        }
                                        ValueType::Nested(nested_schema) => {
                                            Self::write_model(
                                                writer,
                                                &field.xml_name,
                                                item,
                                                nested_schema,
                                            )?;
                                        }
                                        ValueType::List(_) => {}
                                    }
                                }
                            }
                        }
                        ValueType::Nested(nested_schema) => {
                            Self::write_model(writer, &field.xml_name, val, nested_schema)?;
                        }
                    }
                }
            }
        }

        // Close element
        writer
            .write_event(Event::End(BytesEnd::new(std::str::from_utf8(tag_name)?)))
            .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;

        Ok(())
    }
}
