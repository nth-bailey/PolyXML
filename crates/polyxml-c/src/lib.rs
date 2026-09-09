use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

use polyxml_core::schema::{FieldKind, FieldSchema, ModelSchema, ModelSchemaBuilder, ScalarType, ValueType};
use polyxml_core::value::PolyValue;

// Opaque types
pub struct PolyXmlSchemaBuilder(ModelSchemaBuilder);
pub struct PolyXmlSchema(Arc<ModelSchema>);
pub struct PolyXmlValue(PolyValue);

#[repr(C)]
pub enum PolyXmlFieldKind {
    Attribute = 0,
    Element = 1,
    Text = 2,
}

#[repr(C)]
pub enum PolyXmlScalarType {
    String = 0,
    Int = 1,
    Float = 2,
    Bool = 3,
    Decimal = 4,
    XmlDate = 5,
    XmlDateTime = 6,
    Any = 7,
}

#[repr(C)]
pub enum PolyXmlErrorCode {
    Ok = 0,
    ErrSyntax = 1,
    ErrScalar = 2,
    ErrSchema = 3,
    ErrNullPtr = 4,
    ErrUtf8 = 5,
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_schema_builder_create(name: *const c_char) -> *mut PolyXmlSchemaBuilder {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(PolyXmlSchemaBuilder(ModelSchema::builder(name_str))))
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_schema_builder_add_field(
    builder: *mut PolyXmlSchemaBuilder,
    name: *const c_char,
    xml_name: *const c_char,
    kind: PolyXmlFieldKind,
    scalar_type: PolyXmlScalarType,
) {
    if builder.is_null() || name.is_null() || xml_name.is_null() {
        return;
    }
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let xml_bytes = CStr::from_ptr(xml_name).to_bytes();

    let field_kind = match kind {
        PolyXmlFieldKind::Attribute => FieldKind::Attribute,
        PolyXmlFieldKind::Element => FieldKind::Element,
        PolyXmlFieldKind::Text => FieldKind::Text,
    };

    let sc_type = match scalar_type {
        PolyXmlScalarType::String => ScalarType::String,
        PolyXmlScalarType::Int => ScalarType::Int,
        PolyXmlScalarType::Float => ScalarType::Float,
        PolyXmlScalarType::Bool => ScalarType::Bool,
        PolyXmlScalarType::Decimal => ScalarType::Decimal,
        PolyXmlScalarType::XmlDate => ScalarType::XmlDate,
        PolyXmlScalarType::XmlDateTime => ScalarType::XmlDateTime,
        PolyXmlScalarType::Any => ScalarType::Any,
    };

    let field = FieldSchema::new(name_str, xml_bytes, field_kind, ValueType::Scalar(sc_type));
    let b = &mut *builder;
    b.0 = std::mem::replace(&mut b.0, ModelSchema::builder("")).field(field);
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_schema_builder_build(builder: *mut PolyXmlSchemaBuilder) -> *mut PolyXmlSchema {
    if builder.is_null() {
        return std::ptr::null_mut();
    }
    let boxed = Box::from_raw(builder);
    let schema = boxed.0.build();
    Box::into_raw(Box::new(PolyXmlSchema(schema)))
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_schema_free(schema: *mut PolyXmlSchema) {
    if !schema.is_null() {
        drop(Box::from_raw(schema));
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_deserialize(
    data: *const u8,
    len: usize,
    schema: *const PolyXmlSchema,
    out_value: *mut *mut PolyXmlValue,
) -> PolyXmlErrorCode {
    if data.is_null() || schema.is_null() || out_value.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }

    let slice = std::slice::from_raw_parts(data, len);
    let s = &(*schema).0;

    match polyxml_core::deserialize(slice, Arc::clone(s)) {
        Ok(val) => {
            *out_value = Box::into_raw(Box::new(PolyXmlValue(val)));
            PolyXmlErrorCode::Ok
        }
        Err(polyxml_core::error::PolyXmlError::XmlSyntaxError { .. }) => PolyXmlErrorCode::ErrSyntax,
        Err(polyxml_core::error::PolyXmlError::ScalarParseError { .. }) => PolyXmlErrorCode::ErrScalar,
        Err(_) => PolyXmlErrorCode::ErrSchema,
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_serialize(
    root_name: *const c_char,
    value: *const PolyXmlValue,
    schema: *const PolyXmlSchema,
    indent: i32,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> PolyXmlErrorCode {
    if root_name.is_null() || value.is_null() || schema.is_null() || out_bytes.is_null() || out_len.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }

    let root_str = match CStr::from_ptr(root_name).to_str() {
        Ok(s) => s,
        Err(_) => return PolyXmlErrorCode::ErrUtf8,
    };

    let indent_opt = if indent > 0 { Some(indent as usize) } else { None };
    let val = &(*value).0;
    let s = &(*schema).0;

    match polyxml_core::serialize(root_str, val, s, indent_opt) {
        Ok(mut bytes) => {
            bytes.shrink_to_fit();
            *out_len = bytes.len();
            *out_bytes = bytes.as_mut_ptr();
            std::mem::forget(bytes);
            PolyXmlErrorCode::Ok
        }
        Err(_) => PolyXmlErrorCode::ErrSchema,
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_bytes_free(bytes: *mut u8, len: usize) {
    if !bytes.is_null() {
        drop(Vec::from_raw_parts(bytes, len, len));
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_get_field(val: *const PolyXmlValue, key: *const c_char) -> *const PolyXmlValue {
    if val.is_null() || key.is_null() {
        return std::ptr::null();
    }
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };
    match &(*val).0 {
        PolyValue::Object(map) => {
            if let Some(sub_val) = map.get(key_str) {
                // Return borrowed reference as pointer
                sub_val as *const PolyValue as *const PolyXmlValue
            } else {
                std::ptr::null()
            }
        }
        _ => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_get_int(val: *const PolyXmlValue, out_int: *mut i64) -> PolyXmlErrorCode {
    if val.is_null() || out_int.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }
    if let Some(i) = (*val).0.as_i64() {
        *out_int = i;
        PolyXmlErrorCode::Ok
    } else {
        PolyXmlErrorCode::ErrScalar
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_get_float(val: *const PolyXmlValue, out_float: *mut f64) -> PolyXmlErrorCode {
    if val.is_null() || out_float.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }
    if let Some(f) = (*val).0.as_f64() {
        *out_float = f;
        PolyXmlErrorCode::Ok
    } else {
        PolyXmlErrorCode::ErrScalar
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_get_bool(val: *const PolyXmlValue, out_bool: *mut bool) -> PolyXmlErrorCode {
    if val.is_null() || out_bool.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }
    if let Some(b) = (*val).0.as_bool() {
        *out_bool = b;
        PolyXmlErrorCode::Ok
    } else {
        PolyXmlErrorCode::ErrScalar
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_get_string(
    val: *const PolyXmlValue,
    out_str: *mut *const c_char,
    out_len: *mut usize,
) -> PolyXmlErrorCode {
    if val.is_null() || out_str.is_null() || out_len.is_null() {
        return PolyXmlErrorCode::ErrNullPtr;
    }
    if let Some(s) = (*val).0.as_str() {
        *out_str = s.as_ptr() as *const c_char;
        *out_len = s.len();
        PolyXmlErrorCode::Ok
    } else {
        PolyXmlErrorCode::ErrScalar
    }
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_is_null(val: *const PolyXmlValue) -> bool {
    if val.is_null() {
        return true;
    }
    (*val).0.is_null()
}

#[no_mangle]
pub unsafe extern "C" fn polyxml_value_free(val: *mut PolyXmlValue) {
    if !val.is_null() {
        drop(Box::from_raw(val));
    }
}

#[no_mangle]
pub extern "C" fn polyxml_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}
