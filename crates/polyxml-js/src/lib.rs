use napi::bindgen_prelude::*;
use napi::JsUnknown;
use napi_derive::napi;
use std::sync::Arc;

use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::value::PolyValue;

#[napi(object)]
pub struct JsFieldDef {
    pub name: String,
    pub xml_name: String,
    pub kind: String,
    pub scalar_type: String,
}

#[napi(object)]
pub struct JsModelSchema {
    pub name: String,
    pub fields: Vec<JsFieldDef>,
}

fn convert_js_schema(schema: &JsModelSchema) -> Arc<ModelSchema> {
    let mut builder = ModelSchema::builder(&schema.name);
    for f in &schema.fields {
        let kind = match f.kind.as_str() {
            "attribute" => FieldKind::Attribute,
            "text" => FieldKind::Text,
            _ => FieldKind::Element,
        };
        let sc = match f.scalar_type.as_str() {
            "int" => ScalarType::Int,
            "float" => ScalarType::Float,
            "bool" => ScalarType::Bool,
            "decimal" => ScalarType::Decimal,
            "xml_date" => ScalarType::XmlDate,
            "xml_datetime" => ScalarType::XmlDateTime,
            _ => ScalarType::String,
        };
        builder = builder.field(FieldSchema::new(
            &f.name,
            f.xml_name.as_bytes(),
            kind,
            ValueType::Scalar(sc),
        ));
    }
    builder.build()
}

fn poly_value_to_js(env: &Env, val: &PolyValue) -> Result<JsUnknown> {
    match val {
        PolyValue::Null => Ok(env.get_null()?.into_unknown()),
        PolyValue::Bool(b) => Ok(env.get_boolean(*b)?.into_unknown()),
        PolyValue::Int(i) => Ok(env.create_int64(*i)?.into_unknown()),
        PolyValue::Float(f) => Ok(env.create_double(*f)?.into_unknown()),
        PolyValue::String(s) => Ok(env.create_string(s)?.into_unknown()),
        PolyValue::List(list) => {
            let mut arr = env.create_array(list.len() as u32)?;
            for (idx, item) in list.iter().enumerate() {
                let js_item = poly_value_to_js(env, item)?;
                arr.set(idx as u32, js_item)?;
            }
            Ok(arr.coerce_to_object()?.into_unknown())
        }
        PolyValue::Object(map) => {
            let mut obj = env.create_object()?;
            for (k, v) in map {
                let js_val = poly_value_to_js(env, v)?;
                obj.set(k.as_str(), js_val)?;
            }
            Ok(obj.into_unknown())
        }
    }
}

#[napi]
pub fn deserialize(
    env: Env,
    xml: Either<String, Buffer>,
    schema: JsModelSchema,
) -> Result<JsUnknown> {
    let xml_bytes: &[u8] = match &xml {
        Either::A(s) => s.as_bytes(),
        Either::B(b) => b.as_ref(),
    };

    let model_schema = convert_js_schema(&schema);
    let val = polyxml::deserialize(xml_bytes, model_schema)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    poly_value_to_js(&env, &val)
}

fn js_to_poly_value(_env: &Env, val: JsUnknown, scalar_type: &str) -> Result<PolyValue> {
    let val_type = val.get_type()?;
    if val_type == napi::ValueType::Null || val_type == napi::ValueType::Undefined {
        return Ok(PolyValue::Null);
    }
    match scalar_type {
        "int" => {
            let num = val.coerce_to_number()?;
            Ok(PolyValue::Int(num.get_int64()?))
        }
        "float" => {
            let num = val.coerce_to_number()?;
            Ok(PolyValue::Float(num.get_double()?))
        }
        "bool" => {
            let b = val.coerce_to_bool()?;
            Ok(PolyValue::Bool(b.get_value()?))
        }
        _ => {
            let s = val.coerce_to_string()?;
            let utf8 = s.into_utf8()?;
            Ok(PolyValue::String(utf8.as_str()?.to_string()))
        }
    }
}

#[napi]
pub fn serialize(
    env: Env,
    root_name: String,
    value: napi::JsObject,
    schema: JsModelSchema,
    indent: Option<u32>,
) -> Result<Buffer> {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    for f in &schema.fields {
        if value.has_named_property(&f.name)? {
            let prop: JsUnknown = value.get_named_property(&f.name)?;
            let pv = js_to_poly_value(&env, prop, &f.scalar_type)?;
            map.insert(f.name.clone(), pv);
        }
    }
    let poly_val = PolyValue::Object(map);
    let model_schema = convert_js_schema(&schema);
    let indent_opt = indent.map(|i| i as usize);
    let bytes = polyxml::serialize(&root_name, &poly_val, &model_schema, indent_opt)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(bytes.into())
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
