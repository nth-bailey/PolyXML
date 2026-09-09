use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolyXmlError {
    #[error("XML reader error at position {position}: {source}")]
    XmlSyntaxError {
        position: u64,
        #[source]
        source: quick_xml::Error,
    },

    #[error("Invalid UTF-8 in XML document: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Scalar parse error for field '{field}': expected {expected}, got '{value}'")]
    ScalarParseError {
        field: String,
        expected: &'static str,
        value: String,
    },

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Unexpected root element '{actual}', expected '{expected}'")]
    UnexpectedRootElement { expected: String, actual: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, PolyXmlError>;
