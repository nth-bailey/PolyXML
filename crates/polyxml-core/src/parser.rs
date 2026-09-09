use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use smallvec::{smallvec, SmallVec};

use crate::converters::ValueConverter;
use crate::error::{PolyXmlError, Result};
use crate::schema::{ModelSchema, ScalarType, ValueType};
use crate::value::PolyValue;

pub(crate) struct StackFrame {
    schema: Arc<ModelSchema>,
    values: SmallVec<[Option<PolyValue>; 8]>,
    list_values: SmallVec<[Option<Vec<PolyValue>>; 4]>,
    frame_text_buf: Option<Vec<u8>>,
}

impl StackFrame {
    fn new(schema: Arc<ModelSchema>) -> Self {
        let field_count = schema.fields.len();
        let frame_text_buf = if schema.text_field.is_some() {
            Some(Vec::new())
        } else {
            None
        };
        Self {
            schema,
            values: smallvec![None; field_count],
            list_values: SmallVec::new(),
            frame_text_buf,
        }
    }

    #[inline]
    fn push_list_item(&mut self, idx: usize, val: PolyValue) {
        if self.list_values.is_empty() {
            self.list_values
                .resize_with(self.schema.fields.len(), || None);
        }
        if let Some(list_opt) = self.list_values.get_mut(idx) {
            match list_opt {
                Some(list) => list.push(val),
                None => *list_opt = Some(vec![val]),
            }
        }
    }

    fn finish(mut self) -> Result<PolyValue> {
        let mut obj = HashMap::with_capacity(self.schema.fields.len());

        for (idx, field) in self.schema.fields.iter().enumerate() {
            if let Some(val) = self.values.get_mut(idx).and_then(|v| v.take()) {
                obj.insert(field.name.clone(), val);
            }
        }

        if !self.list_values.is_empty() {
            for (idx, list_opt) in self.list_values.into_iter().enumerate() {
                if let Some(list) = list_opt {
                    let field = &self.schema.fields[idx];
                    obj.insert(field.name.clone(), PolyValue::List(list));
                }
            }
        }

        if let Some(text_idx) = self.schema.text_field {
            if let Some(ref text_buf) = self.frame_text_buf {
                if !text_buf.is_empty() {
                    let field = &self.schema.fields[text_idx];
                    if let ValueType::Scalar(ref st) = field.val_type {
                        let val = ValueConverter::parse_scalar(st, text_buf, &field.name)?;
                        obj.insert(field.name.clone(), val);
                    }
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
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    return Self::parse_sub_tree(&mut reader, root_schema, e, max_depth);
                }
                Ok(Event::Empty(ref e)) => {
                    let mut frame = StackFrame::new(Arc::clone(&root_schema));
                    Self::parse_attributes(e, &mut frame)?;
                    return frame.finish();
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

    pub(crate) fn parse_sub_tree<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        root_schema: Arc<ModelSchema>,
        root_start: &BytesStart,
        max_depth: usize,
    ) -> Result<PolyValue> {
        let mut stack: Vec<StackFrame> = Vec::with_capacity(16);
        let mut root_frame = StackFrame::new(Arc::clone(&root_schema));
        Self::parse_attributes(root_start, &mut root_frame)?;
        stack.push(root_frame);

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
                    let local_name = e.local_name();
                    let is_nil = is_nil_element(e);

                    let current_schema = Arc::clone(&stack.last().unwrap().schema);
                    if let Some(&field_idx) = current_schema.element_map.get(local_name.as_ref()) {
                        let field = &current_schema.fields[field_idx];
                        match &field.val_type {
                            ValueType::Scalar(st) => {
                                if is_nil {
                                    stack.last_mut().unwrap().values[field_idx] =
                                        Some(PolyValue::Null);
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
                                            .push_list_item(field_idx, PolyValue::Null);
                                    } else {
                                        active_scalar_field = Some((field_idx, st.clone(), true));
                                        text_buf.clear();
                                    }
                                }
                                ValueType::Nested(sub_schema) => {
                                    let mut frame = StackFrame::new(Arc::clone(sub_schema));
                                    Self::parse_attributes(e, &mut frame)?;
                                    stack.push(frame);
                                }
                                _ => {}
                            },
                            ValueType::Nested(sub_schema) => {
                                let mut frame = StackFrame::new(Arc::clone(sub_schema));
                                Self::parse_attributes(e, &mut frame)?;
                                stack.push(frame);
                            }
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name();
                    let is_nil = is_nil_element(e);

                    let current_schema = Arc::clone(&stack.last().unwrap().schema);
                    if let Some(&field_idx) = current_schema.element_map.get(local_name.as_ref()) {
                        let field = &current_schema.fields[field_idx];
                        match &field.val_type {
                            ValueType::Scalar(_) => {
                                let val = if is_nil {
                                    PolyValue::Null
                                } else {
                                    PolyValue::String(String::new())
                                };
                                stack.last_mut().unwrap().values[field_idx] = Some(val);
                            }
                            ValueType::List(inner) => match inner.as_ref() {
                                ValueType::Scalar(_) => {
                                    let val = if is_nil {
                                        PolyValue::Null
                                    } else {
                                        PolyValue::String(String::new())
                                    };
                                    stack.last_mut().unwrap().push_list_item(field_idx, val);
                                }
                                ValueType::Nested(sub_schema) => {
                                    let mut frame = StackFrame::new(Arc::clone(sub_schema));
                                    Self::parse_attributes(e, &mut frame)?;
                                    let instance = frame.finish()?;
                                    stack
                                        .last_mut()
                                        .unwrap()
                                        .push_list_item(field_idx, instance);
                                }
                                _ => {}
                            },
                            ValueType::Nested(sub_schema) => {
                                let mut frame = StackFrame::new(Arc::clone(sub_schema));
                                Self::parse_attributes(e, &mut frame)?;
                                let instance = frame.finish()?;
                                stack.last_mut().unwrap().values[field_idx] = Some(instance);
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let unescaped = e.unescape().map_err(PolyXmlError::XmlError)?;
                    if active_scalar_field.is_some() {
                        text_buf.extend_from_slice(unescaped.as_bytes());
                    } else if let Some(frame) = stack.last_mut() {
                        if let Some(ref mut tb) = frame.frame_text_buf {
                            tb.extend_from_slice(unescaped.as_bytes());
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if active_scalar_field.is_some() {
                        text_buf.extend_from_slice(e.as_ref());
                    } else if let Some(frame) = stack.last_mut() {
                        if let Some(ref mut tb) = frame.frame_text_buf {
                            tb.extend_from_slice(e.as_ref());
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
                            frame.push_list_item(field_idx, parsed_val);
                        } else {
                            frame.values[field_idx] = Some(parsed_val);
                        }
                    } else if stack.len() > 1 {
                        let finished_frame = stack.pop().unwrap();
                        let local_name = e.local_name();
                        let instance = finished_frame.finish()?;
                        let parent = stack.last_mut().unwrap();

                        if let Some(&field_idx) = parent.schema.element_map.get(local_name.as_ref())
                        {
                            let field = &parent.schema.fields[field_idx];
                            if matches!(field.val_type, ValueType::List(_)) {
                                parent.push_list_item(field_idx, instance);
                            } else {
                                parent.values[field_idx] = Some(instance);
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

    pub(crate) fn parse_attributes(e: &BytesStart, frame: &mut StackFrame) -> Result<()> {
        for attr in e.attributes().flatten() {
            let key = attr.key.local_name();
            if let Some(&field_idx) = frame.schema.attribute_map.get(key.as_ref()) {
                let field = &frame.schema.fields[field_idx];
                if let ValueType::Scalar(ref st) = field.val_type {
                    let unescaped = attr.unescape_value().map_err(PolyXmlError::XmlError)?;
                    let val = ValueConverter::parse_scalar(st, unescaped.as_bytes(), &field.name)?;
                    frame.values[field_idx] = Some(val);
                }
            }
        }
        Ok(())
    }
}

pub struct XmlItemStream<R: std::io::BufRead> {
    reader: Reader<R>,
    schema: Arc<ModelSchema>,
    target_tag: Vec<u8>,
    max_depth: usize,
    buf: Vec<u8>,
}

impl<R: std::io::BufRead> XmlItemStream<R> {
    pub fn new(reader: R, schema: Arc<ModelSchema>, target_tag: &[u8]) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        Self {
            reader: xml_reader,
            schema,
            target_tag: target_tag.to_vec(),
            max_depth: DEFAULT_MAX_DEPTH,
            buf: Vec::new(),
        }
    }

    pub fn next_item(&mut self) -> Result<Option<PolyValue>> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == self.target_tag.as_slice() {
                        let item = XmlDeserializer::parse_sub_tree(
                            &mut self.reader,
                            Arc::clone(&self.schema),
                            e,
                            self.max_depth,
                        )?;
                        return Ok(Some(item));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == self.target_tag.as_slice() {
                        let mut frame = StackFrame::new(Arc::clone(&self.schema));
                        XmlDeserializer::parse_attributes(e, &mut frame)?;
                        return Ok(Some(frame.finish()?));
                    }
                }
                Ok(Event::Eof) => return Ok(None),
                Err(err) => {
                    return Err(PolyXmlError::XmlSyntaxError {
                        position: self.reader.buffer_position(),
                        source: err,
                    });
                }
                _ => {}
            }
        }
    }
}
