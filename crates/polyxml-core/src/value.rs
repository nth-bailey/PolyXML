use std::collections::HashMap;
use std::sync::Arc;

use crate::schema::ModelSchema;

#[derive(Debug, Clone)]
pub enum PolyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<PolyValue>),
    Object(HashMap<String, PolyValue>),
    Record {
        schema: Arc<ModelSchema>,
        values: Box<[Option<PolyValue>]>,
    },
}

impl PartialEq for PolyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PolyValue::Null, PolyValue::Null) => true,
            (PolyValue::Bool(a), PolyValue::Bool(b)) => a == b,
            (PolyValue::Int(a), PolyValue::Int(b)) => a == b,
            (PolyValue::Float(a), PolyValue::Float(b)) => a == b,
            (PolyValue::String(a), PolyValue::String(b)) => a == b,
            (PolyValue::List(a), PolyValue::List(b)) => a == b,
            (PolyValue::Object(a), PolyValue::Object(b)) => a == b,
            (
                PolyValue::Record {
                    schema: s1,
                    values: v1,
                },
                PolyValue::Record {
                    schema: s2,
                    values: v2,
                },
            ) => (Arc::ptr_eq(s1, s2) || s1.name == s2.name) && v1 == v2,
            (PolyValue::Record { schema, values }, PolyValue::Object(map))
            | (PolyValue::Object(map), PolyValue::Record { schema, values }) => {
                for (idx, field) in schema.fields.iter().enumerate() {
                    let rec_val = values.get(idx).and_then(|v| v.as_ref());
                    let map_val = map.get(&field.name);
                    if rec_val != map_val {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl PolyValue {
    pub fn is_null(&self) -> bool {
        matches!(self, PolyValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PolyValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PolyValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PolyValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PolyValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[PolyValue]> {
        match self {
            PolyValue::List(l) => Some(l.as_slice()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, PolyValue>> {
        match self {
            PolyValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_record(&self) -> Option<(&Arc<ModelSchema>, &[Option<PolyValue>])> {
        match self {
            PolyValue::Record { schema, values } => Some((schema, values.as_ref())),
            _ => None,
        }
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&PolyValue> {
        match self {
            PolyValue::Record { values, .. } => values.get(idx).and_then(|v| v.as_ref()),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&PolyValue> {
        match self {
            PolyValue::Object(o) => o.get(key),
            PolyValue::Record { schema, values } => {
                let idx = schema.fields.iter().position(|f| f.name == key)?;
                values.get(idx).and_then(|v| v.as_ref())
            }
            _ => None,
        }
    }

    pub fn to_hash_map(&self) -> Option<HashMap<String, PolyValue>> {
        match self {
            PolyValue::Object(o) => Some(o.clone()),
            PolyValue::Record { schema, values } => {
                let mut map = HashMap::with_capacity(schema.fields.len());
                for (idx, field) in schema.fields.iter().enumerate() {
                    if let Some(Some(val)) = values.get(idx) {
                        map.insert(field.name.clone(), val.clone());
                    }
                }
                Some(map)
            }
            _ => None,
        }
    }

    /// Serialize this PolyValue into raw JSON bytes.
    pub fn to_json(&self, indent: Option<usize>) -> crate::error::Result<Vec<u8>> {
        let json_val = serde_json::Value::from(self);
        match indent {
            Some(spaces) if spaces > 0 => {
                let indent_str = " ".repeat(spaces);
                let formatter =
                    serde_json::ser::PrettyFormatter::with_indent(indent_str.as_bytes());
                let mut buf = Vec::new();
                let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                serde::Serialize::serialize(&json_val, &mut ser)?;
                Ok(buf)
            }
            _ => Ok(serde_json::to_vec(&json_val)?),
        }
    }

    /// Deserialize raw JSON bytes into a dynamic PolyValue without a schema.
    pub fn from_json(json_bytes: &[u8]) -> crate::error::Result<Self> {
        let json_val: serde_json::Value = serde_json::from_slice(json_bytes)?;
        Ok(PolyValue::from(json_val))
    }
}
