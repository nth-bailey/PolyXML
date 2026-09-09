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
            kind,
            val_type,
            required: false,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ModelSchema {
    pub name: String,
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
    fields: Vec<FieldSchema>,
}

impl ModelSchemaBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
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

        Arc::new(ModelSchema {
            name: self.name,
            fields: self.fields,
            element_map,
            attribute_map,
            text_field,
        })
    }
}
