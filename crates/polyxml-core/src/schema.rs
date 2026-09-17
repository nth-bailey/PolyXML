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
