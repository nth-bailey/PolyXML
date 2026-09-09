use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use polyxml_core::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml_core::value::PolyValue;

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
            Ok(arr.into_unknown())
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
    let val = polyxml_core::deserialize(xml_bytes, model_schema)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    poly_value_to_js(&env, &val)
}

#[napi]
pub fn version() -> &'static str {
    "0.1.0"
}
