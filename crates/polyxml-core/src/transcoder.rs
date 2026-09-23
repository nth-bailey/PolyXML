use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use serde_json::{Map, Value};

use crate::error::{PolyXmlError, Result};
use crate::json::{deserialize_json, serialize_json};
use crate::parser::XmlDeserializer;
use crate::schema::ModelSchema;
use crate::serializer::XmlSerializer;

/// Transcode XML bytes into JSON bytes.
///
/// If a `ModelSchema` is provided, typed data-binding is used, respecting numeric, boolean,
/// and collection types as well as field aliases.
/// If `schema` is None, XML events are assembled into a JSON value tree.
/// The complete input and output are held in memory.
pub fn xml_to_json(
    xml: &[u8],
    schema: Option<Arc<ModelSchema>>,
    indent: Option<usize>,
    by_alias: bool,
) -> Result<Vec<u8>> {
    if let Some(s) = schema {
        let value = XmlDeserializer::deserialize(xml, Arc::clone(&s))?;
        serialize_json(&value, &s, indent, by_alias)
    } else {
        xml_to_json_dynamic(xml, indent)
    }
}

/// Transcode JSON bytes into XML bytes.
///
/// If a `ModelSchema` is provided, typed data-binding is used, mapping JSON properties to XML
/// elements, attributes, and namespaces.
/// If `schema` is None, the complete JSON input is parsed into a value tree
/// before XML output is written into a byte buffer.
pub fn json_to_xml(
    json: &[u8],
    schema: Option<Arc<ModelSchema>>,
    root_name: Option<&str>,
    indent: Option<usize>,
    namespaces: Option<bool>,
    ns_map: Option<&HashMap<String, String>>,
) -> Result<Vec<u8>> {
    if let Some(s) = schema {
        let value = deserialize_json(json, Arc::clone(&s))?;
        let effective_root = root_name.unwrap_or(&s.name);
        XmlSerializer::serialize_with_options(
            effective_root,
            &value,
            &s,
            indent,
            namespaces,
            ns_map,
        )
    } else {
        json_to_xml_dynamic(json, root_name, indent)
    }
}

struct DynamicFrame {
    tag_name: String,
    attrs: Map<String, Value>,
    children: Vec<(String, Value)>,
    text_buf: String,
}

fn parse_dynamic_scalar(s: &str) -> Value {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Value::String(String::new());
    }
    if trimmed == "true" {
        return Value::Bool(true);
    }
    if trimmed == "false" {
        return Value::Bool(false);
    }
    if let Ok(n) = trimmed.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            return Value::Number(num);
        }
    }
    Value::String(s.to_string())
}

/// Schema-less dynamic XML -> JSON transcoder.
pub fn xml_to_json_dynamic(xml: &[u8], indent: Option<usize>) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(Cursor::new(xml));
    // Do NOT enable `trim_text`: quick-xml trims *each* Text event, and text is
    // split at entity references, so `x &amp; y` would become `x&y` — losing
    // interior spaces around the reference. `End` trims the assembled buffer
    // instead, which is the only correct place to normalize whitespace.

    let mut buf = Vec::new();
    let mut stack: Vec<DynamicFrame> = Vec::new();
    let mut root_val: Option<(String, Value)> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = e.local_name().as_ref().to_string();
                let mut attrs = Map::new();
                for attr in e.attributes() {
                    let attr = attr.map_err(|err| PolyXmlError::XmlSyntaxError {
                        position: reader.buffer_position(),
                        source: quick_xml::Error::InvalidAttr(err),
                    })?;
                    let key = attr.key.local_name().as_ref().to_string();
                    let raw_val = attr.value.as_ref();
                    let unescaped = quick_xml::escape::unescape(raw_val).map_err(|err| {
                        PolyXmlError::XmlSyntaxError {
                            position: reader.buffer_position(),
                            source: quick_xml::Error::Escape(err),
                        }
                    })?;
                    attrs.insert(format!("@{}", key), parse_dynamic_scalar(&unescaped));
                }
                stack.push(DynamicFrame {
                    tag_name: tag,
                    attrs,
                    children: Vec::new(),
                    text_buf: String::new(),
                });
            }
            Ok(Event::Text(ref e)) => {
                if let Some(frame) = stack.last_mut() {
                    let text = quick_xml::escape::unescape(e.as_ref()).map_err(|err| {
                        PolyXmlError::XmlSyntaxError {
                            position: reader.buffer_position(),
                            source: quick_xml::Error::Escape(err),
                        }
                    })?;
                    frame.text_buf.push_str(&text);
                }
            }
            Ok(Event::CData(ref e)) => {
                if let Some(frame) = stack.last_mut() {
                    frame.text_buf.push_str(e.as_ref());
                }
            }
            Ok(Event::GeneralRef(ref e)) => {
                if let Some(frame) = stack.last_mut() {
                    if e.is_char_ref() {
                        if let Some(ch) = e.resolve_char_ref().map_err(PolyXmlError::XmlError)? {
                            frame.text_buf.push(ch);
                        }
                    } else if let Some(val) = quick_xml::escape::resolve_xml_entity(e.as_ref()) {
                        frame.text_buf.push_str(val);
                    } else {
                        frame.text_buf.push_str(e.as_ref());
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = e.local_name().as_ref().to_string();
                let mut attrs = Map::new();
                for attr in e.attributes() {
                    let attr = attr.map_err(|err| PolyXmlError::XmlSyntaxError {
                        position: reader.buffer_position(),
                        source: quick_xml::Error::InvalidAttr(err),
                    })?;
                    let key = attr.key.local_name().as_ref().to_string();
                    let raw_val = attr.value.as_ref();
                    let unescaped = quick_xml::escape::unescape(raw_val).map_err(|err| {
                        PolyXmlError::XmlSyntaxError {
                            position: reader.buffer_position(),
                            source: quick_xml::Error::Escape(err),
                        }
                    })?;
                    attrs.insert(format!("@{}", key), parse_dynamic_scalar(&unescaped));
                }
                let val = if attrs.is_empty() {
                    Value::Null
                } else {
                    Value::Object(attrs)
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push((tag, val));
                } else {
                    root_val = Some((tag, val));
                }
            }
            Ok(Event::End(_)) => {
                if let Some(frame) = stack.pop() {
                    let mut obj = frame.attrs;
                    let has_children = !frame.children.is_empty();
                    let has_attrs = !obj.is_empty();
                    let trimmed_text = frame.text_buf.trim();
                    let has_text = !trimmed_text.is_empty();

                    let node_val = if !has_children && !has_attrs {
                        if has_text {
                            parse_dynamic_scalar(trimmed_text)
                        } else {
                            Value::String(String::new())
                        }
                    } else {
                        // Group repeated children into JSON arrays
                        let mut order = Vec::new();
                        let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
                        for (child_name, child_val) in frame.children {
                            if !groups.contains_key(&child_name) {
                                order.push(child_name.clone());
                            }
                            groups.entry(child_name).or_default().push(child_val);
                        }

                        for child_name in order {
                            let mut items = groups.remove(&child_name).unwrap();
                            if items.len() == 1 {
                                obj.insert(child_name, items.pop().unwrap());
                            } else {
                                obj.insert(child_name, Value::Array(items));
                            }
                        }

                        if has_text {
                            obj.insert("value".to_string(), parse_dynamic_scalar(trimmed_text));
                        }

                        Value::Object(obj)
                    };

                    if let Some(parent) = stack.last_mut() {
                        parent.children.push((frame.tag_name, node_val));
                    } else {
                        root_val = Some((frame.tag_name, node_val));
                    }
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

    let Some((root_tag, val)) = root_val else {
        return Err(PolyXmlError::SchemaError("Empty XML input".into()));
    };

    let mut top = Map::new();
    top.insert(root_tag, val);
    let final_json = Value::Object(top);

    let bytes = if let Some(indent_size) = indent {
        let indent_str = " ".repeat(indent_size);
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_str.as_bytes());
        let mut out_buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut out_buf, formatter);
        serde::Serialize::serialize(&final_json, &mut ser)
            .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?;
        out_buf
    } else {
        serde_json::to_vec(&final_json)
            .map_err(|e| PolyXmlError::SerializationError(e.to_string()))?
    };

    Ok(bytes)
}

/// Schema-less dynamic JSON -> XML transcoder.
pub fn json_to_xml_dynamic(
    json_bytes: &[u8],
    root_name: Option<&str>,
    indent: Option<usize>,
) -> Result<Vec<u8>> {
    let val: Value = serde_json::from_slice(json_bytes)
        .map_err(|e| PolyXmlError::SerializationError(format!("Invalid JSON: {}", e)))?;

    let mut out = Vec::new();
    let mut writer = Writer::new(Cursor::new(&mut out));

    match &val {
        Value::Object(map) if root_name.is_none() && map.len() == 1 => {
            let (root_tag, inner_val) = map.iter().next().unwrap();
            emit_json_value_as_xml(&mut writer, root_tag, inner_val, 0, indent)?;
        }
        _ => {
            let root_tag = root_name.unwrap_or("root");
            emit_json_value_as_xml(&mut writer, root_tag, &val, 0, indent)?;
        }
    }

    if indent.is_some() {
        out.push(b'\n');
    }

    Ok(out)
}

fn emit_indent<W: std::io::Write>(
    writer: &mut Writer<W>,
    depth: usize,
    indent: Option<usize>,
) -> Result<()> {
    if let Some(indent_size) = indent {
        writer.write_event(Event::Text(BytesText::new("\n")))?;
        let spaces = " ".repeat(depth * indent_size);
        if !spaces.is_empty() {
            writer.write_event(Event::Text(BytesText::new(&spaces)))?;
        }
    }
    Ok(())
}

fn emit_json_value_as_xml<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag_name: &str,
    val: &Value,
    depth: usize,
    indent: Option<usize>,
) -> Result<()> {
    emit_indent(writer, depth, indent)?;

    match val {
        Value::Null => {
            let start = BytesStart::new(tag_name);
            writer.write_event(Event::Empty(start))?;
        }
        Value::Bool(b) => {
            let start = BytesStart::new(tag_name);
            writer.write_event(Event::Start(start))?;
            let s = if *b { "true" } else { "false" };
            writer.write_event(Event::Text(BytesText::new(s)))?;
            writer.write_event(Event::End(BytesEnd::new(tag_name)))?;
        }
        Value::Number(n) => {
            let start = BytesStart::new(tag_name);
            writer.write_event(Event::Start(start))?;
            let s = n.to_string();
            writer.write_event(Event::Text(BytesText::new(&s)))?;
            writer.write_event(Event::End(BytesEnd::new(tag_name)))?;
        }
        Value::String(s) => {
            let start = BytesStart::new(tag_name);
            writer.write_event(Event::Start(start))?;
            writer.write_event(Event::Text(BytesText::new(s)))?;
            writer.write_event(Event::End(BytesEnd::new(tag_name)))?;
        }
        Value::Array(arr) => {
            for item in arr {
                emit_json_value_as_xml(writer, tag_name, item, depth, indent)?;
            }
        }
        Value::Object(map) => {
            let mut start = BytesStart::new(tag_name);
            let mut children = Vec::new();
            let mut text_val = None;

            for (k, v) in map {
                if let Some(attr_name) = k.strip_prefix('@') {
                    let attr_str = match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    start.push_attribute((attr_name, attr_str.as_str()));
                } else if k == "value" || k == "#text" || k == "$" {
                    text_val = Some(match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    });
                } else {
                    children.push((k.as_str(), v));
                }
            }

            if children.is_empty() && text_val.is_none() {
                writer.write_event(Event::Empty(start))?;
            } else {
                writer.write_event(Event::Start(start))?;
                for (child_name, child_val) in &children {
                    if let Value::Array(items) = child_val {
                        for item in items {
                            emit_json_value_as_xml(writer, child_name, item, depth + 1, indent)?;
                        }
                    } else {
                        emit_json_value_as_xml(writer, child_name, child_val, depth + 1, indent)?;
                    }
                }
                if let Some(t) = text_val {
                    writer.write_event(Event::Text(BytesText::new(&t)))?;
                }
                if !children.is_empty() {
                    emit_indent(writer, depth, indent)?;
                }
                writer.write_event(Event::End(BytesEnd::new(tag_name)))?;
            }
        }
    }

    Ok(())
}
