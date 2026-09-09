use crate::error::{PolyXmlError, Result};
use crate::schema::ScalarType;
use crate::value::PolyValue;

#[inline(always)]
pub fn trim_bytes(mut b: &[u8]) -> &[u8] {
    while let Some((first, rest)) = b.split_first() {
        if first.is_ascii_whitespace() {
            b = rest;
        } else {
            break;
        }
    }
    while let Some((last, rest)) = b.split_last() {
        if last.is_ascii_whitespace() {
            b = rest;
        } else {
            break;
        }
    }
    b
}

pub struct ValueConverter;

impl ValueConverter {
    pub fn parse_scalar(scalar_type: &ScalarType, bytes: &[u8], field_name: &str) -> Result<PolyValue> {
        match scalar_type {
            ScalarType::String => {
                let s = std::str::from_utf8(bytes)?;
                Ok(PolyValue::String(s.to_string()))
            }
            ScalarType::Int => {
                let trimmed = trim_bytes(bytes);
                let val: i64 = lexical_core::parse(trimmed).map_err(|_| PolyXmlError::ScalarParseError {
                    field: field_name.to_string(),
                    expected: "integer",
                    value: String::from_utf8_lossy(bytes).to_string(),
                })?;
                Ok(PolyValue::Int(val))
            }
            ScalarType::Float => {
                let trimmed = trim_bytes(bytes);
                let val: f64 = lexical_core::parse(trimmed).map_err(|_| PolyXmlError::ScalarParseError {
                    field: field_name.to_string(),
                    expected: "float",
                    value: String::from_utf8_lossy(bytes).to_string(),
                })?;
                Ok(PolyValue::Float(val))
            }
            ScalarType::Bool => {
                let trimmed = trim_bytes(bytes);
                match trimmed {
                    b"true" | b"1" => Ok(PolyValue::Bool(true)),
                    b"false" | b"0" => Ok(PolyValue::Bool(false)),
                    _ => Err(PolyXmlError::ScalarParseError {
                        field: field_name.to_string(),
                        expected: "boolean ('true', 'false', '1', '0')",
                        value: String::from_utf8_lossy(bytes).to_string(),
                    }),
                }
            }
            ScalarType::Decimal
            | ScalarType::XmlDate
            | ScalarType::XmlDateTime
            | ScalarType::XmlTime
            | ScalarType::XmlDuration
            | ScalarType::Any => {
                let s = std::str::from_utf8(trim_bytes(bytes))?;
                Ok(PolyValue::String(s.to_string()))
            }
        }
    }
}
