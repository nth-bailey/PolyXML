use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PolyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<PolyValue>),
    Object(HashMap<String, PolyValue>),
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

    pub fn get(&self, key: &str) -> Option<&PolyValue> {
        match self {
            PolyValue::Object(o) => o.get(key),
            _ => None,
        }
    }
}
