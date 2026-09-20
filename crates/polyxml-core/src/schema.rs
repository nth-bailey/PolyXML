use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Attribute,
    Element,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarType {
    String,
    Int,
    Float,
    Bool,
    Decimal,
    XmlDate,
    XmlDateTime,
    XmlTime,
    XmlDuration,
    Any,
}

#[derive(Debug, Clone)]
pub enum ValueType {
    Scalar(ScalarType),
    List(Box<ValueType>),
    Nested(Arc<ModelSchema>),
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub xml_name: Vec<u8>,
    pub namespace: Option<String>,
    pub kind: FieldKind,
    pub val_type: ValueType,
    pub required: bool,
}

impl FieldSchema {
    pub fn new(
        name: impl Into<String>,
        xml_name: &[u8],
        kind: FieldKind,
        val_type: ValueType,
    ) -> Self {
        Self {
            name: name.into(),
            xml_name: xml_name.to_vec(),
            namespace: None,
            kind,
            val_type,
            required: false,
        }
    }

    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    pub fn with_namespace(self, ns: impl Into<String>) -> Self {
        self.namespace(ns)
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ModelSchema {
    pub name: String,
    pub xml_name: Vec<u8>,
    pub namespace: Option<String>,
    pub fields: Vec<FieldSchema>,
    pub element_map: HashMap<Vec<u8>, usize>,
    pub attribute_map: HashMap<Vec<u8>, usize>,
    pub text_field: Option<usize>,
}

impl ModelSchema {
    pub fn builder(name: impl Into<String>) -> ModelSchemaBuilder {
        ModelSchemaBuilder::new(name)
    }

    /// Construct a runtime `ModelSchema` from a compiled `SchemaIR`.
    pub fn from_ir(
        ir: &crate::ir::SchemaIR,
        root_name: Option<&str>,
    ) -> crate::error::Result<Arc<ModelSchema>> {
        use crate::ir::{PrimitiveType, TypeDef, TypeRef};
        use std::collections::HashSet;

        let (name, qname_opt, type_ref) = if let Some(target) = root_name {
            if let Some((qname, elem)) = ir.elements.iter().find(|(q, _)| q.local == target) {
                (
                    elem.qname.local.clone(),
                    Some(qname.clone()),
                    elem.type_ref.clone(),
                )
            } else if let Some((qname, _)) = ir.types.iter().find(|(q, _)| q.local == target) {
                (
                    qname.local.clone(),
                    Some(qname.clone()),
                    TypeRef::Named(qname.clone()),
                )
            } else {
                return Err(crate::error::PolyXmlError::SchemaError(format!(
                    "Root element or type '{}' not found in schema",
                    target
                )));
            }
        } else if let Some((_, elem)) = ir.elements.iter().next() {
            (
                elem.qname.local.clone(),
                Some(elem.qname.clone()),
                elem.type_ref.clone(),
            )
        } else if let Some((qname, _)) = ir.types.iter().next() {
            (
                qname.local.clone(),
                Some(qname.clone()),
                TypeRef::Named(qname.clone()),
            )
        } else {
            return Err(crate::error::PolyXmlError::SchemaError(
                "SchemaIR contains no elements or types".into(),
            ));
        };

        fn map_primitive(prim: PrimitiveType) -> ScalarType {
            match prim {
                PrimitiveType::String
                | PrimitiveType::NormalizedString
                | PrimitiveType::Token
                | PrimitiveType::Language
                | PrimitiveType::Name
                | PrimitiveType::NCName
                | PrimitiveType::Id
                | PrimitiveType::IdRef
                | PrimitiveType::IdRefs
                | PrimitiveType::Entity
                | PrimitiveType::Entities
                | PrimitiveType::NMTOKEN
                | PrimitiveType::NMTOKENS
                | PrimitiveType::AnyUri
                | PrimitiveType::QName => ScalarType::String,
                PrimitiveType::Boolean => ScalarType::Bool,
                PrimitiveType::Decimal => ScalarType::Decimal,
                PrimitiveType::Float | PrimitiveType::Double => ScalarType::Float,
                PrimitiveType::Duration => ScalarType::XmlDuration,
                PrimitiveType::DateTime => ScalarType::XmlDateTime,
                PrimitiveType::Time => ScalarType::XmlTime,
                PrimitiveType::Date => ScalarType::XmlDate,
                PrimitiveType::Int
                | PrimitiveType::Integer
                | PrimitiveType::NonPositiveInteger
                | PrimitiveType::NegativeInteger
                | PrimitiveType::Long
                | PrimitiveType::Short
                | PrimitiveType::Byte
                | PrimitiveType::NonNegativeInteger
                | PrimitiveType::UnsignedLong
                | PrimitiveType::UnsignedInt
                | PrimitiveType::UnsignedShort
                | PrimitiveType::UnsignedByte
                | PrimitiveType::PositiveInteger => ScalarType::Int,
                PrimitiveType::Base64Binary | PrimitiveType::HexBinary => ScalarType::String,
                _ => ScalarType::String,
            }
        }

        fn build_type(
            tr: &TypeRef,
            ir: &crate::ir::SchemaIR,
            visited: &mut HashSet<crate::ir::QName>,
        ) -> ValueType {
            match tr {
                TypeRef::Primitive(prim) => ValueType::Scalar(map_primitive(*prim)),
                TypeRef::List(inner) => ValueType::List(Box::new(build_type(inner, ir, visited))),
                TypeRef::Boxed(inner) => build_type(inner, ir, visited),
                TypeRef::Named(qname) => {
                    if let Some(type_def) = ir.types.get(qname) {
                        match type_def {
                            TypeDef::Struct(s) => {
                                if visited.contains(&s.qname) {
                                    ValueType::Nested(ModelSchema::builder(&s.qname.local).build())
                                } else {
                                    visited.insert(s.qname.clone());
                                    let child_schema = build_struct(s, ir, visited);
                                    visited.remove(&s.qname);
                                    ValueType::Nested(child_schema)
                                }
                            }
                            TypeDef::Simple(sim) => build_type(&sim.base_type, ir, visited),
                            TypeDef::Enum(_) => ValueType::Scalar(ScalarType::String),
                            TypeDef::Union(_) => ValueType::Scalar(ScalarType::String),
                        }
                    } else {
                        ValueType::Scalar(ScalarType::String)
                    }
                }
            }
        }

        fn build_struct(
            s: &crate::ir::StructDef,
            ir: &crate::ir::SchemaIR,
            visited: &mut HashSet<crate::ir::QName>,
        ) -> Arc<ModelSchema> {
            let mut builder = ModelSchema::builder(&s.qname.local);
            if let Some(ref ns) = s.qname.namespace {
                builder = builder.namespace(ns);
            }

            for f in &s.fields {
                let kind = match f.kind {
                    crate::ir::FieldKind::Attribute => FieldKind::Attribute,
                    crate::ir::FieldKind::Element => FieldKind::Element,
                    crate::ir::FieldKind::Text => FieldKind::Text,
                    _ => FieldKind::Element,
                };

                let mut val_type = build_type(&f.type_ref, ir, visited);
                if f.cardinality.is_list() && !matches!(val_type, ValueType::List(_)) {
                    val_type = ValueType::List(Box::new(val_type));
                }

                let mut field_schema =
                    FieldSchema::new(&f.name, f.xml_name.as_bytes(), kind, val_type);
                if let Some(ref ns) = f.namespace {
                    field_schema = field_schema.namespace(ns);
                }
                if f.cardinality.min_occurs > 0 && !f.cardinality.is_optional() {
                    field_schema = field_schema.required();
                }

                builder = builder.field(field_schema);
            }

            builder.build()
        }

        let mut visited = HashSet::new();
        if let Some(qn) = qname_opt.as_ref() {
            visited.insert(qn.clone());
        }

        match type_ref {
            TypeRef::Named(ref qname) if ir.types.contains_key(qname) => {
                if let Some(TypeDef::Struct(s)) = ir.types.get(qname) {
                    Ok(build_struct(s, ir, &mut visited))
                } else {
                    let mut b = ModelSchema::builder(name);
                    b = b.field(FieldSchema::new(
                        "value",
                        b"value",
                        FieldKind::Text,
                        build_type(&type_ref, ir, &mut visited),
                    ));
                    Ok(b.build())
                }
            }
            _ => {
                let mut b = ModelSchema::builder(name);
                b = b.field(FieldSchema::new(
                    "value",
                    b"value",
                    FieldKind::Text,
                    build_type(&type_ref, ir, &mut visited),
                ));
                Ok(b.build())
            }
        }
    }
}

pub struct ModelSchemaBuilder {
    name: String,
    xml_name: Option<Vec<u8>>,
    namespace: Option<String>,
    fields: Vec<FieldSchema>,
}

impl ModelSchemaBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            xml_name: None,
            namespace: None,
            fields: Vec::new(),
        }
    }

    pub fn xml_name(mut self, xml_name: &[u8]) -> Self {
        self.xml_name = Some(xml_name.to_vec());
        self
    }

    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    pub fn with_namespace(self, ns: impl Into<String>) -> Self {
        self.namespace(ns)
    }

    pub fn field(mut self, field: FieldSchema) -> Self {
        self.fields.push(field);
        self
    }

    pub fn build(self) -> Arc<ModelSchema> {
        let mut element_map = HashMap::new();
        let mut attribute_map = HashMap::new();
        let mut text_field = None;

        for (idx, field) in self.fields.iter().enumerate() {
            match field.kind {
                FieldKind::Attribute => {
                    attribute_map.insert(field.xml_name.clone(), idx);
                    // Match local name if prefix exists (e.g., "prefix:attr" -> "attr")
                    if let Some(pos) = field.xml_name.iter().position(|&b| b == b':') {
                        attribute_map.insert(field.xml_name[pos + 1..].to_vec(), idx);
                    }
                }
                FieldKind::Element => {
                    element_map.insert(field.xml_name.clone(), idx);
                    if let Some(pos) = field.xml_name.iter().position(|&b| b == b':') {
                        element_map.insert(field.xml_name[pos + 1..].to_vec(), idx);
                    }
                }
                FieldKind::Text => {
                    text_field = Some(idx);
                }
            }
        }

        let xml_name = self
            .xml_name
            .unwrap_or_else(|| self.name.as_bytes().to_vec());

        Arc::new(ModelSchema {
            name: self.name,
            xml_name,
            namespace: self.namespace,
            fields: self.fields,
            element_map,
            attribute_map,
            text_field,
        })
    }
}
