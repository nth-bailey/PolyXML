#![allow(clippy::type_complexity)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyString, PyType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use polyxml::schema::{FieldKind, FieldSchema, ModelSchema, ScalarType, ValueType};
use polyxml::value::PolyValue;

struct CachedSchemaMeta {
    schema: Arc<ModelSchema>,
    py_cls: PyObject,
    is_dataclass: bool,
    field_pystrings: Vec<Py<PyString>>,
}

// Global thread-safe schema cache keyed by Python type pointer
static SCHEMA_CACHE: RwLock<Option<HashMap<usize, Arc<CachedSchemaMeta>>>> = RwLock::new(None);
// Cache keyed by schema name to resolve nested Python types during deserialization
static CLASS_BY_SCHEMA: RwLock<Option<HashMap<String, Arc<CachedSchemaMeta>>>> = RwLock::new(None);

fn lookup_cached_meta(schema_name: &str) -> Option<Arc<CachedSchemaMeta>> {
    let class_map = CLASS_BY_SCHEMA.read().unwrap_or_else(|p| p.into_inner());
    if let Some(ref map) = *class_map {
        if let Some(meta) = map.get(schema_name) {
            return Some(Arc::clone(meta));
        }
    }
    None
}

fn lookup_py_class<'py>(py: Python<'py>, schema_name: &str) -> Option<Bound<'py, PyAny>> {
    lookup_cached_meta(schema_name).map(|meta| meta.py_cls.bind(py).clone())
}

fn resolve_scalar_type(py: Python<'_>, type_obj: &Bound<'_, PyAny>) -> PyResult<ScalarType> {
    let type_name: String = type_obj
        .getattr("__name__")
        .and_then(|n| n.extract())
        .unwrap_or_default();

    match type_name.as_str() {
        "str" => Ok(ScalarType::String),
        "int" => Ok(ScalarType::Int),
        "float" => Ok(ScalarType::Float),
        "bool" => Ok(ScalarType::Bool),
        "Decimal" => Ok(ScalarType::Decimal),
        "XmlDate" => Ok(ScalarType::XmlDate),
        "XmlDateTime" => Ok(ScalarType::XmlDateTime),
        "XmlTime" => Ok(ScalarType::XmlTime),
        "XmlDuration" => Ok(ScalarType::XmlDuration),
        _ => {
            if let Ok(py_type) = type_obj.downcast::<PyType>() {
                if let Ok(enum_module) = py.import_bound("enum") {
                    if let Ok(enum_cls) = enum_module.getattr("Enum") {
                        if let Ok(py_enum) = enum_cls.downcast::<PyType>() {
                            if py_type.is_subclass(py_enum).unwrap_or(false) {
                                return Ok(ScalarType::Any);
                            }
                        }
                    }
                }
            }
            Ok(ScalarType::Any)
        }
    }
}

fn resolve_value_type(py: Python<'_>, type_obj: &Bound<'_, PyAny>) -> PyResult<ValueType> {
    // Handle typing.Optional / Union
    if let Ok(origin) = type_obj.getattr("__origin__") {
        if let Ok(origin_name) = origin.getattr("__name__") {
            let origin_str: String = origin_name.extract().unwrap_or_default();
            if origin_str == "Union" || origin_str == "UnionType" {
                if let Ok(args) = type_obj.getattr("__args__") {
                    let tuple: Bound<'_, pyo3::types::PyTuple> = args.downcast_into()?;
                    for arg in tuple.iter() {
                        let arg_name: String = arg
                            .getattr("__name__")
                            .and_then(|n| n.extract())
                            .unwrap_or_default();
                        if arg_name != "NoneType" {
                            return resolve_value_type(py, &arg);
                        }
                    }
                }
            } else if origin_str == "list" || origin_str == "List" {
                if let Ok(args) = type_obj.getattr("__args__") {
                    let tuple: Bound<'_, pyo3::types::PyTuple> = args.downcast_into()?;
                    if let Some(first) = tuple.iter().next() {
                        let inner = resolve_value_type(py, &first)?;
                        return Ok(ValueType::List(Box::new(inner)));
                    }
                }
                return Ok(ValueType::List(Box::new(ValueType::Scalar(
                    ScalarType::String,
                ))));
            }
        }
    }

    // Check if target is a nested dataclass or Pydantic model
    if type_obj.hasattr("__dataclass_fields__")? || type_obj.hasattr("model_fields")? {
        if let Ok(py_type) = type_obj.downcast::<PyType>() {
            let (nested_schema, _) = get_or_create_schema(py_type)?;
            return Ok(ValueType::Nested(nested_schema));
        }
    }

    let scalar = resolve_scalar_type(py, type_obj)?;
    Ok(ValueType::Scalar(scalar))
}

fn extract_schema_from_class<'py>(cls: &Bound<'py, PyType>) -> PyResult<Arc<ModelSchema>> {
    let py = cls.py();
    let class_name: String = cls.getattr("__name__")?.extract()?;
    let mut builder = ModelSchema::builder(class_name);

    if cls.hasattr("model_fields")? {
        let model_fields: Bound<'py, PyDict> = cls.getattr("model_fields")?.downcast_into()?;
        for (name_obj, field_obj) in model_fields.iter() {
            let py_name: String = name_obj.extract()?;
            let mut xml_name = py_name.clone();
            let mut kind = FieldKind::Element;

            let meta = field_obj
                .getattr("xsdata_metadata")
                .ok()
                .or_else(|| field_obj.getattr("json_schema_extra").ok());

            if let Some(ref m) = meta {
                if let Ok(t) = m.get_item("type") {
                    let t_str: String = t.extract().unwrap_or_default();
                    match t_str.as_str() {
                        "Attribute" => kind = FieldKind::Attribute,
                        "Text" => kind = FieldKind::Text,
                        _ => kind = FieldKind::Element,
                    }
                }
                if let Ok(n) = m.get_item("name") {
                    xml_name = n.extract().unwrap_or(py_name.clone());
                }
            }

            let field_type = field_obj.getattr("annotation")?;
            let val_type = resolve_value_type(py, &field_type)?;
            builder = builder.field(FieldSchema::new(
                py_name,
                xml_name.as_bytes(),
                kind,
                val_type,
            ));
        }
    } else if cls.hasattr("__dataclass_fields__")? {
        let fields: Bound<'py, PyDict> = cls.getattr("__dataclass_fields__")?.downcast_into()?;
        for (name_obj, field_obj) in fields.iter() {
            let py_name: String = name_obj.extract()?;
            let mut xml_name = py_name.clone();
            let mut kind = FieldKind::Element;

            if let Ok(meta) = field_obj.getattr("metadata") {
                if let Ok(t) = meta.get_item("type") {
                    let t_str: String = t.extract().unwrap_or_default();
                    match t_str.as_str() {
                        "Attribute" => kind = FieldKind::Attribute,
                        "Text" => kind = FieldKind::Text,
                        _ => kind = FieldKind::Element,
                    }
                }
                if let Ok(n) = meta.get_item("name") {
                    xml_name = n.extract().unwrap_or(py_name.clone());
                }
            }

            let field_type = field_obj.getattr("type")?;
            let val_type = resolve_value_type(py, &field_type)?;
            builder = builder.field(FieldSchema::new(
                py_name,
                xml_name.as_bytes(),
                kind,
                val_type,
            ));
        }
    }

    Ok(builder.build())
}

fn get_or_create_schema_meta<'py>(cls: &Bound<'py, PyType>) -> PyResult<Arc<CachedSchemaMeta>> {
    let type_key = cls.as_ptr() as usize;

    {
        let cache = SCHEMA_CACHE.read().unwrap_or_else(|p| p.into_inner());
        if let Some(ref map) = *cache {
            if let Some(meta) = map.get(&type_key) {
                return Ok(Arc::clone(meta));
            }
        }
    }

    let is_dataclass = cls.hasattr("__dataclass_fields__")? && !cls.hasattr("model_fields")?;
    let schema = extract_schema_from_class(cls)?;
    let py = cls.py();
    let py_cls_obj: PyObject = cls.clone().into_any().unbind();

    let mut field_pystrings = Vec::with_capacity(schema.fields.len());
    for field in &schema.fields {
        field_pystrings.push(PyString::new_bound(py, &field.name).unbind());
    }

    let meta = Arc::new(CachedSchemaMeta {
        schema: Arc::clone(&schema),
        py_cls: py_cls_obj,
        is_dataclass,
        field_pystrings,
    });

    {
        let mut cache = SCHEMA_CACHE.write().unwrap_or_else(|p| p.into_inner());
        let map = cache.get_or_insert_with(HashMap::new);
        map.insert(type_key, Arc::clone(&meta));
    }

    {
        let mut class_map = CLASS_BY_SCHEMA.write().unwrap_or_else(|p| p.into_inner());
        let map = class_map.get_or_insert_with(HashMap::new);
        map.insert(schema.name.clone(), Arc::clone(&meta));
    }

    Ok(meta)
}

fn get_or_create_schema<'py>(cls: &Bound<'py, PyType>) -> PyResult<(Arc<ModelSchema>, PyObject)> {
    let meta = get_or_create_schema_meta(cls)?;
    Ok((Arc::clone(&meta.schema), meta.py_cls.clone_ref(cls.py())))
}

fn poly_value_to_py<'py>(
    py: Python<'py>,
    val: &PolyValue,
    val_type: &ValueType,
    cls: Option<&Bound<'py, PyAny>>,
) -> PyResult<PyObject> {
    match val {
        PolyValue::Null => Ok(py.None()),
        PolyValue::Bool(b) => Ok(b.to_object(py)),
        PolyValue::Int(i) => Ok(i.to_object(py)),
        PolyValue::Float(f) => Ok(f.to_object(py)),
        PolyValue::String(s) => Ok(PyString::new_bound(py, s).into_any().unbind()),
        PolyValue::List(items) => {
            if let ValueType::List(inner_type) = val_type {
                let inner_cls = if let ValueType::Nested(ref s) = inner_type.as_ref() {
                    lookup_py_class(py, &s.name)
                } else {
                    None
                };
                let mut py_items = Vec::with_capacity(items.len());
                for item in items {
                    let py_item = poly_value_to_py(py, item, inner_type, inner_cls.as_ref())?;
                    py_items.push(py_item);
                }
                let py_list = PyList::new_bound(py, &py_items);
                Ok(py_list.into_any().unbind())
            } else {
                let py_list = PyList::empty_bound(py);
                Ok(py_list.into_any().unbind())
            }
        }
        PolyValue::Object(map) => {
            let meta_opt = if let ValueType::Nested(ref s) = val_type {
                lookup_cached_meta(&s.name)
            } else {
                None
            };

            let effective_cls = match cls {
                Some(c) => Some(c.clone()),
                None => meta_opt.as_ref().map(|m| m.py_cls.bind(py).clone()),
            };

            if let Some(ref target_cls) = effective_cls {
                if let ValueType::Nested(ref schema) = val_type {
                    // Method 1: Fast Positional Tuple Construction for Dataclasses when all fields are present
                    if let Some(ref meta) = meta_opt {
                        if meta.is_dataclass && schema.fields.len() == map.len() {
                            let mut args_vec = Vec::with_capacity(schema.fields.len());
                            let mut all_found = true;
                            for field in &schema.fields {
                                if let Some(field_val) = map.get(&field.name) {
                                    let field_cls = if let ValueType::Nested(ref s) = field.val_type
                                    {
                                        lookup_py_class(py, &s.name)
                                    } else {
                                        None
                                    };
                                    let py_val = poly_value_to_py(
                                        py,
                                        field_val,
                                        &field.val_type,
                                        field_cls.as_ref(),
                                    )?;
                                    args_vec.push(py_val);
                                } else {
                                    all_found = false;
                                    break;
                                }
                            }
                            if all_found {
                                let tuple = pyo3::types::PyTuple::new_bound(py, &args_vec);
                                let instance = target_cls.call1(tuple)?;
                                return Ok(instance.unbind());
                            }
                        }
                    }

                    // Fallback to keyword arguments using pre-interned PyString keys
                    let kwargs = PyDict::new_bound(py);
                    for (i, field) in schema.fields.iter().enumerate() {
                        if let Some(field_val) = map.get(&field.name) {
                            let field_cls = if let ValueType::Nested(ref s) = field.val_type {
                                lookup_py_class(py, &s.name)
                            } else {
                                None
                            };
                            let py_val = poly_value_to_py(
                                py,
                                field_val,
                                &field.val_type,
                                field_cls.as_ref(),
                            )?;
                            if let Some(ref meta) = meta_opt {
                                if i < meta.field_pystrings.len() {
                                    kwargs.set_item(meta.field_pystrings[i].bind(py), py_val)?;
                                    continue;
                                }
                            }
                            kwargs.set_item(&field.name, py_val)?;
                        }
                    }
                    let instance = target_cls.call((), Some(&kwargs))?;
                    return Ok(instance.unbind());
                }
                let dict = PyDict::new_bound(py);
                for (k, v) in map {
                    let py_v = poly_value_to_py(py, v, &ValueType::Scalar(ScalarType::Any), None)?;
                    dict.set_item(k, py_v)?;
                }
                let instance = target_cls.call((), Some(&dict))?;
                Ok(instance.unbind())
            } else {
                let dict = PyDict::new_bound(py);
                for (k, v) in map {
                    let py_v = poly_value_to_py(py, v, &ValueType::Scalar(ScalarType::Any), None)?;
                    dict.set_item(k, py_v)?;
                }
                Ok(dict.into_any().unbind())
            }
        }
    }
}

fn py_to_poly_value<'py>(
    py: Python<'py>,
    obj: &Bound<'py, PyAny>,
    schema: &ModelSchema,
    meta_opt: Option<&CachedSchemaMeta>,
) -> PyResult<PolyValue> {
    let mut map = HashMap::with_capacity(schema.fields.len());

    for (i, field) in schema.fields.iter().enumerate() {
        let val_res = if let Some(meta) = meta_opt {
            if i < meta.field_pystrings.len() {
                obj.getattr(meta.field_pystrings[i].bind(py))
            } else {
                obj.getattr(field.name.as_str())
            }
        } else {
            obj.getattr(field.name.as_str())
        };

        if let Ok(val) = val_res {
            if val.is_none() {
                map.insert(field.name.clone(), PolyValue::Null);
                continue;
            }

            match &field.val_type {
                ValueType::Scalar(st) => match st {
                    ScalarType::Int => {
                        let i: i64 = val.extract()?;
                        map.insert(field.name.clone(), PolyValue::Int(i));
                    }
                    ScalarType::Float => {
                        let f: f64 = val.extract()?;
                        map.insert(field.name.clone(), PolyValue::Float(f));
                    }
                    ScalarType::Bool => {
                        let b: bool = val.extract()?;
                        map.insert(field.name.clone(), PolyValue::Bool(b));
                    }
                    _ => {
                        let s: String = val.str()?.extract()?;
                        map.insert(field.name.clone(), PolyValue::String(s));
                    }
                },
                ValueType::List(inner) => {
                    if let Ok(list) = val.downcast::<PyList>() {
                        let mut poly_items = Vec::with_capacity(list.len());
                        let child_meta = if let ValueType::Nested(sub_schema) = inner.as_ref() {
                            lookup_cached_meta(&sub_schema.name)
                        } else {
                            None
                        };
                        for item in list.iter() {
                            if let ValueType::Nested(sub_schema) = inner.as_ref() {
                                poly_items.push(py_to_poly_value(
                                    py,
                                    &item,
                                    sub_schema,
                                    child_meta.as_deref(),
                                )?);
                            } else {
                                let s: String = item.str()?.extract()?;
                                poly_items.push(PolyValue::String(s));
                            }
                        }
                        map.insert(field.name.clone(), PolyValue::List(poly_items));
                    }
                }
                ValueType::Nested(sub_schema) => {
                    let child_meta = lookup_cached_meta(&sub_schema.name);
                    map.insert(
                        field.name.clone(),
                        py_to_poly_value(py, &val, sub_schema, child_meta.as_deref())?,
                    );
                }
            }
        }
    }

    Ok(PolyValue::Object(map))
}

#[pyfunction]
fn deserialize<'py>(
    py: Python<'py>,
    source: &[u8],
    target_type: Bound<'py, PyType>,
) -> PyResult<PyObject> {
    let meta = get_or_create_schema_meta(&target_type)?;
    let poly_val = polyxml::deserialize(source, Arc::clone(&meta.schema))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let bound_cls = meta.py_cls.bind(py);
    poly_value_to_py(
        py,
        &poly_val,
        &ValueType::Nested(Arc::clone(&meta.schema)),
        Some(bound_cls),
    )
}

#[pyfunction]
#[pyo3(signature = (source, target_type, tag=None))]
fn iterparse<'py>(
    py: Python<'py>,
    source: &[u8],
    target_type: Bound<'py, PyType>,
    tag: Option<String>,
) -> PyResult<XmlIterator> {
    let meta = get_or_create_schema_meta(&target_type)?;
    let target_tag = tag.unwrap_or_else(|| meta.schema.name.clone());
    let stream = polyxml::XmlItemStream::new(
        std::io::Cursor::new(source.to_vec()),
        Arc::clone(&meta.schema),
        target_tag.as_bytes(),
    );
    Ok(XmlIterator {
        stream,
        schema: Arc::clone(&meta.schema),
        py_cls: meta.py_cls.clone_ref(py),
    })
}

#[pyclass]
struct XmlIterator {
    stream: polyxml::XmlItemStream<std::io::Cursor<Vec<u8>>>,
    schema: Arc<ModelSchema>,
    py_cls: PyObject,
}

#[pymethods]
impl XmlIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(mut slf: PyRefMut<'py, Self>, py: Python<'py>) -> PyResult<Option<PyObject>> {
        match slf.stream.next_item() {
            Ok(Some(poly_val)) => {
                let schema = Arc::clone(&slf.schema);
                let target_cls = slf.py_cls.clone_ref(py);
                let bound_cls = target_cls.bind(py);
                let py_obj =
                    poly_value_to_py(py, &poly_val, &ValueType::Nested(schema), Some(bound_cls))?;
                Ok(Some(py_obj))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn serialize<'py>(
    py: Python<'py>,
    obj: Bound<'py, PyAny>,
    indent: Option<usize>,
) -> PyResult<Bound<'py, PyBytes>> {
    let cls = obj.get_type();
    let meta = get_or_create_schema_meta(&cls)?;

    let poly_val = py_to_poly_value(py, &obj, &meta.schema, Some(&meta))?;
    let root_name = meta.schema.name.as_str();

    let bytes = polyxml::serialize(root_name, &poly_val, &meta.schema, indent)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyBytes::new_bound(py, &bytes))
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _polyxml(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<XmlIterator>()?;
    m.add_function(wrap_pyfunction!(deserialize, m)?)?;
    m.add_function(wrap_pyfunction!(iterparse, m)?)?;
    m.add_function(wrap_pyfunction!(serialize, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
