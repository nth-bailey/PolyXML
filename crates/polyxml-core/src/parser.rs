use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::converters::ValueConverter;
use crate::error::{PolyXmlError, Result};
use crate::schema::{ModelSchema, ScalarType, ValueType};
use crate::value::PolyValue;

struct StackFrame {
    schema: Arc<ModelSchema>,
    #[allow(dead_code)]
    element_name: Vec<u8>,
    scalar_values: HashMap<usize, PolyValue>,
    list_values: HashMap<usize, Vec<PolyValue>>,
    frame_text_buf: Vec<u8>,
}

impl StackFrame {
    fn new(schema: Arc<ModelSchema>, element_name: Vec<u8>) -> Self {
        Self {
            schema,
            element_name,
            scalar_values: HashMap::new(),
            list_values: HashMap::new(),
            frame_text_buf: Vec::new(),
        }
    }

    fn finish(self) -> Result<PolyValue> {
        let mut obj = HashMap::with_capacity(self.schema.fields.len());

        for (idx, val) in self.scalar_values {
            let field = &self.schema.fields[idx];
            obj.insert(field.name.clone(), val);
        }

        for (idx, list_items) in self.list_values {
            let field = &self.schema.fields[idx];
            obj.insert(field.name.clone(), PolyValue::List(list_items));
        }

        if let Some(text_idx) = self.schema.text_field {
            if !self.frame_text_buf.is_empty() {
                let field = &self.schema.fields[text_idx];
                if let ValueType::Scalar(ref st) = field.val_type {
                    let val = ValueConverter::parse_scalar(st, &self.frame_text_buf, &field.name)?;
                    obj.insert(field.name.clone(), val);
                }
            }
        }

        Ok(PolyValue::Object(obj))
    }
}

pub struct XmlDeserializer;

fn is_nil_element(e: &BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        let key = attr.key.local_name();
        if (key.as_ref() == b"nil" || key.as_ref() == b"xsi:nil")
            && (attr.value.as_ref() == b"true" || attr.value.as_ref() == b"1")
        {
            return true;
        }
    }
    false
}

pub const DEFAULT_MAX_DEPTH: usize = 256;

impl XmlDeserializer {
    pub fn deserialize(xml_bytes: &[u8], root_schema: Arc<ModelSchema>) -> Result<PolyValue> {
        Self::deserialize_with_limit(xml_bytes, root_schema, DEFAULT_MAX_DEPTH)
    }

    pub fn deserialize_with_limit(
        xml_bytes: &[u8],
        root_schema: Arc<ModelSchema>,
        max_depth: usize,
    ) -> Result<PolyValue> {
        let mut reader = Reader::from_reader(xml_bytes);
        reader.config_mut().trim_text(true);

        let mut stack: Vec<StackFrame> = Vec::with_capacity(16);
        let mut active_scalar_field: Option<(usize, ScalarType, bool)> = None;
        let mut text_buf: Vec<u8> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if stack.len() >= max_depth {
                        return Err(PolyXmlError::MaxDepthExceeded {
                            max_depth,
                            current: stack.len() + 1,
                        });
                    }
                    let local_name = e.local_name().as_ref().to_vec();
                    let is_nil = is_nil_element(e);

                    if stack.is_empty() {
                        let mut frame = StackFrame::new(Arc::clone(&root_schema), local_name);
                        Self::parse_attributes(e, &mut frame)?;
                        stack.push(frame);
                    } else {
                        let current_schema = Arc::clone(&stack.last().unwrap().schema);
                        if let Some(&field_idx) = current_schema.element_map.get(&local_name) {
                            let field = &current_schema.fields[field_idx];
                            match &field.val_type {
                                ValueType::Scalar(st) => {
                                    if is_nil {
                                        stack
                                            .last_mut()
                                            .unwrap()
                                            .scalar_values
                                            .insert(field_idx, PolyValue::Null);
                                    } else {
                                        active_scalar_field = Some((field_idx, st.clone(), false));
                                        text_buf.clear();
                                    }
                                }
                                ValueType::List(inner) => match inner.as_ref() {
                                    ValueType::Scalar(st) => {
                                        if is_nil {
                                            stack
                                                .last_mut()
                                                .unwrap()
                                                .list_values
                                                .entry(field_idx)
                                                .or_default()
                                                .push(PolyValue::Null);
                                        } else {
                                            active_scalar_field =
                                                Some((field_idx, st.clone(), true));
                                            text_buf.clear();
                                        }
                                    }
                                    ValueType::Nested(sub_schema) => {
                                        let mut frame =
                                            StackFrame::new(Arc::clone(sub_schema), local_name);
                                        Self::parse_attributes(e, &mut frame)?;
                                        stack.push(frame);
                                    }
                                    _ => {}
                                },
                                ValueType::Nested(sub_schema) => {
                                    let mut frame =
                                        StackFrame::new(Arc::clone(sub_schema), local_name);
                                    Self::parse_attributes(e, &mut frame)?;
                                    stack.push(frame);
                                }
                            }
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name().as_ref().to_vec();
                    let is_nil = is_nil_element(e);

                    if stack.is_empty() {
                        let mut frame = StackFrame::new(Arc::clone(&root_schema), local_name);
                        Self::parse_attributes(e, &mut frame)?;
                        return frame.finish();
                    } else {
                        let current_schema = Arc::clone(&stack.last().unwrap().schema);
                        if let Some(&field_idx) = current_schema.element_map.get(&local_name) {
                            let field = &current_schema.fields[field_idx];
                            match &field.val_type {
                                ValueType::Scalar(_) => {
                                    let val = if is_nil {
                                        PolyValue::Null
                                    } else {
                                        PolyValue::String(String::new())
                                    };
                                    stack
                                        .last_mut()
                                        .unwrap()
                                        .scalar_values
                                        .insert(field_idx, val);
                                }
                                ValueType::List(inner) => match inner.as_ref() {
                                    ValueType::Scalar(_) => {
                                        let val = if is_nil {
                                            PolyValue::Null
                                        } else {
                                            PolyValue::String(String::new())
                                        };
                                        stack
                                            .last_mut()
                                            .unwrap()
                                            .list_values
                                            .entry(field_idx)
                                            .or_default()
                                            .push(val);
                                    }
                                    ValueType::Nested(sub_schema) => {
                                        let mut frame =
                                            StackFrame::new(Arc::clone(sub_schema), local_name);
                                        Self::parse_attributes(e, &mut frame)?;
                                        let instance = frame.finish()?;
                                        stack
                                            .last_mut()
                                            .unwrap()
                                            .list_values
                                            .entry(field_idx)
                                            .or_default()
                                            .push(instance);
                                    }
                                    _ => {}
                                },
                                ValueType::Nested(sub_schema) => {
                                    let mut frame =
                                        StackFrame::new(Arc::clone(sub_schema), local_name);
                                    Self::parse_attributes(e, &mut frame)?;
                                    let instance = frame.finish()?;
                                    stack
                                        .last_mut()
                                        .unwrap()
                                        .scalar_values
                                        .insert(field_idx, instance);
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let unescaped = e.unescape().map_err(PolyXmlError::XmlError)?;
                    if active_scalar_field.is_some() {
                        text_buf.extend_from_slice(unescaped.as_bytes());
                    } else if let Some(frame) = stack.last_mut() {
                        if frame.schema.text_field.is_some() {
                            frame.frame_text_buf.extend_from_slice(unescaped.as_bytes());
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if active_scalar_field.is_some() {
                        text_buf.extend_from_slice(e.as_ref());
                    } else if let Some(frame) = stack.last_mut() {
                        if frame.schema.text_field.is_some() {
                            frame.frame_text_buf.extend_from_slice(e.as_ref());
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if let Some((field_idx, ref scalar_type, is_list)) = active_scalar_field.take()
                    {
                        let field_name = &stack.last().unwrap().schema.fields[field_idx].name;
                        let parsed_val =
                            ValueConverter::parse_scalar(scalar_type, &text_buf, field_name)?;
                        let frame = stack.last_mut().unwrap();
                        if is_list {
                            frame
                                .list_values
                                .entry(field_idx)
                                .or_default()
                                .push(parsed_val);
                        } else {
                            frame.scalar_values.insert(field_idx, parsed_val);
                        }
                    } else if stack.len() > 1 {
                        let finished_frame = stack.pop().unwrap();
                        let local_name = e.local_name().as_ref().to_vec();
                        let instance = finished_frame.finish()?;
                        let parent = stack.last_mut().unwrap();

                        if let Some(&field_idx) = parent.schema.element_map.get(&local_name) {
                            let field = &parent.schema.fields[field_idx];
                            if matches!(field.val_type, ValueType::List(_)) {
                                parent
                                    .list_values
                                    .entry(field_idx)
                                    .or_default()
                                    .push(instance);
                            } else {
                                parent.scalar_values.insert(field_idx, instance);
                            }
                        }
                    } else if stack.len() == 1 {
                        let root_frame = stack.pop().unwrap();
                        return root_frame.finish();
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => {
                    return Err(PolyXmlError::XmlSyntaxError {
                        position: reader.buffer_position(),
                        source: err,
                    });
                }
                _ => {}
            }
            buf.clear();
        }

        Err(PolyXmlError::SchemaError(
            "Unexpected end of XML stream".to_string(),
        ))
    }

    fn parse_attributes(e: &BytesStart, frame: &mut StackFrame) -> Result<()> {
        for attr in e.attributes().flatten() {
            let key = attr.key.local_name();
            if let Some(&field_idx) = frame.schema.attribute_map.get(key.as_ref()) {
                let field = &frame.schema.fields[field_idx];
                if let ValueType::Scalar(ref st) = field.val_type {
                    let unescaped = attr.unescape_value().map_err(PolyXmlError::XmlError)?;
                    let val = ValueConverter::parse_scalar(st, unescaped.as_bytes(), &field.name)?;
                    frame.scalar_values.insert(field_idx, val);
                }
            }
        }
        Ok(())
    }
}
