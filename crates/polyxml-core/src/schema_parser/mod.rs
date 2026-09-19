use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use thiserror::Error;

use crate::ir::{
    Cardinality, ElementDef, EnumDef, EnumValue, FieldDef, FieldKind, OccursLimit, PrimitiveType,
    QName, RestrictionFacets, SchemaIR, SimpleTypeDef, StructDef, TypeDef, TypeRef, UnionBranch,
    UnionDef,
};

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Malformed schema: {0}")]
    Malformed(String),

    #[error("Resolution error: {0}")]
    Resolution(String),
}

/// A pure-Rust XSD 1.0/1.1 Schema Parser.
pub struct XsdParser {
    visited_files: HashSet<PathBuf>,
}

impl Default for XsdParser {
    fn default() -> Self {
        Self::new()
    }
}

impl XsdParser {
    pub fn new() -> Self {
        Self {
            visited_files: HashSet::new(),
        }
    }

    /// Parse an XSD schema from a file path, recursively resolving includes and imports.
    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> Result<SchemaIR, SchemaError> {
        let path = path.as_ref();
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !self.visited_files.insert(canonical.clone()) {
            // Already parsed this file
            return Ok(SchemaIR::new());
        }

        let content = fs::read_to_string(path)?;
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        self.parse_str_internal(&content, Some(base_dir))
    }

    /// Parse an XSD schema from a string slice.
    pub fn parse_str(&mut self, xml: &str) -> Result<SchemaIR, SchemaError> {
        self.parse_str_internal(xml, None)
    }

    fn parse_str_internal(
        &mut self,
        xml: &str,
        base_dir: Option<&Path>,
    ) -> Result<SchemaIR, SchemaError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut ir = SchemaIR::new();
        let mut target_namespace = None;
        let mut prefixes = HashMap::new();
        let mut buf = Vec::new();

        // Pass 1: Parse root schema attributes and build prefix table
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    let name = e.name().into_inner();
                    let local = strip_prefix(name);
                    if local == "schema" {
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            let val = attr.value.as_ref();

                            if key == "targetNamespace" {
                                target_namespace = Some(val.to_string());
                                ir.target_namespace = Some(val.to_string());
                            } else if key == "xmlns" {
                                prefixes.insert(String::new(), val.to_string());
                            } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                                prefixes.insert(prefix.to_string(), val.to_string());
                            }
                        }
                        break;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        // Pass 2: Ingest top-level constructs, includes, and imports
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        buf.clear();

        while let Ok(event) = reader.read_event_into(&mut buf) {
            match event {
                Event::Start(ref e) => {
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "include" => {
                            if let Some(schema_location) = get_attr_value(e, "schemaLocation") {
                                if let Some(dir) = base_dir {
                                    let inc_path = dir.join(&schema_location);
                                    if inc_path.exists() {
                                        let sub_ir = self.parse_file(&inc_path)?;
                                        merge_ir(&mut ir, sub_ir);
                                    }
                                }
                            }
                        }
                        "import" => {
                            if let Some(schema_location) = get_attr_value(e, "schemaLocation") {
                                if let Some(dir) = base_dir {
                                    let imp_path = dir.join(&schema_location);
                                    if imp_path.exists() {
                                        let sub_ir = self.parse_file(&imp_path)?;
                                        merge_ir(&mut ir, sub_ir);
                                    }
                                }
                            }
                        }
                        "complexType" => {
                            if let Some(type_def) = self.parse_complex_type(
                                &mut reader,
                                e,
                                target_namespace.as_deref(),
                                &prefixes,
                            )? {
                                ir.add_type(type_def);
                            }
                        }
                        "simpleType" => {
                            if let Some(type_def) = self.parse_simple_type(
                                &mut reader,
                                e,
                                target_namespace.as_deref(),
                                &prefixes,
                            )? {
                                ir.add_type(type_def);
                            }
                        }
                        "element" => {
                            if let Some(elem_def) = self.parse_global_element(
                                &mut reader,
                                e,
                                target_namespace.as_deref(),
                                &prefixes,
                                &mut ir,
                            )? {
                                ir.add_element(elem_def);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Empty(ref e) => {
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "include" => {
                            if let Some(schema_location) = get_attr_value(e, "schemaLocation") {
                                if let Some(dir) = base_dir {
                                    let inc_path = dir.join(&schema_location);
                                    if inc_path.exists() {
                                        let sub_ir = self.parse_file(&inc_path)?;
                                        merge_ir(&mut ir, sub_ir);
                                    }
                                }
                            }
                        }
                        "import" => {
                            if let Some(schema_location) = get_attr_value(e, "schemaLocation") {
                                if let Some(dir) = base_dir {
                                    let imp_path = dir.join(&schema_location);
                                    if imp_path.exists() {
                                        let sub_ir = self.parse_file(&imp_path)?;
                                        merge_ir(&mut ir, sub_ir);
                                    }
                                }
                            }
                        }
                        "element" => {
                            if let Some(elem_def) = parse_empty_global_element(
                                e,
                                target_namespace.as_deref(),
                                &prefixes,
                            ) {
                                ir.add_element(elem_def);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        // Run cycle detection and inject cut points
        ir.resolve_cycles();

        Ok(ir)
    }

    fn parse_complex_type(
        &self,
        reader: &mut Reader<&[u8]>,
        start: &BytesStart,
        target_ns: Option<&str>,
        prefixes: &HashMap<String, String>,
    ) -> Result<Option<TypeDef>, SchemaError> {
        let name = match get_attr_value(start, "name") {
            Some(n) => n,
            None => return Ok(None), // Anonymous type handled in place
        };

        let is_abstract = get_attr_value(start, "abstract")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let qname = QName::new(target_ns, name);
        let mut fields = Vec::new();
        let mut base_type = None;
        let mut documentation = None;
        let mut is_choice_model = false;
        let mut choice_branches = Vec::new();
        let mut buf = Vec::new();

        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    depth += 1;
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "documentation" => {
                            let text = reader.read_text(e.name())?.to_string();
                            documentation = Some(text);
                            depth -= 1;
                        }
                        "extension" => {
                            if let Some(base) = get_attr_value(e, "base") {
                                base_type = Some(resolve_qname(&base, target_ns, prefixes));
                            }
                        }
                        "choice" => {
                            // If direct child or main compositor is choice, record choice branches
                            is_choice_model = true;
                        }
                        "element" => {
                            if let Some(field) =
                                parse_element_field(e, target_ns, prefixes, is_choice_model)
                            {
                                if is_choice_model {
                                    choice_branches.push(UnionBranch {
                                        variant_name: field.name.clone(),
                                        xml_name: field.xml_name.clone(),
                                        namespace: field.namespace.clone(),
                                        type_ref: field.type_ref.clone(),
                                        documentation: field.documentation.clone(),
                                    });
                                }
                                fields.push(field);
                            }
                        }
                        "attribute" => {
                            if let Some(field) = parse_attribute_field(e, target_ns, prefixes) {
                                fields.push(field);
                            }
                        }
                        "any" => {
                            fields.push(parse_any_field(e));
                        }
                        _ => {}
                    }
                }
                Event::Empty(ref e) => {
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "extension" => {
                            if let Some(base) = get_attr_value(e, "base") {
                                base_type = Some(resolve_qname(&base, target_ns, prefixes));
                            }
                        }
                        "element" => {
                            if let Some(field) =
                                parse_element_field(e, target_ns, prefixes, is_choice_model)
                            {
                                if is_choice_model {
                                    choice_branches.push(UnionBranch {
                                        variant_name: field.name.clone(),
                                        xml_name: field.xml_name.clone(),
                                        namespace: field.namespace.clone(),
                                        type_ref: field.type_ref.clone(),
                                        documentation: field.documentation.clone(),
                                    });
                                }
                                fields.push(field);
                            }
                        }
                        "attribute" => {
                            if let Some(field) = parse_attribute_field(e, target_ns, prefixes) {
                                fields.push(field);
                            }
                        }
                        "any" => {
                            fields.push(parse_any_field(e));
                        }
                        _ => {}
                    }
                }
                Event::End(_) => {
                    depth -= 1;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        if is_choice_model && !choice_branches.is_empty() && fields.len() == choice_branches.len() {
            Ok(Some(TypeDef::Union(UnionDef {
                qname,
                branches: choice_branches,
                documentation,
            })))
        } else {
            Ok(Some(TypeDef::Struct(StructDef {
                qname,
                base_type,
                is_abstract,
                fields,
                documentation,
            })))
        }
    }

    fn parse_simple_type(
        &self,
        reader: &mut Reader<&[u8]>,
        start: &BytesStart,
        target_ns: Option<&str>,
        prefixes: &HashMap<String, String>,
    ) -> Result<Option<TypeDef>, SchemaError> {
        let name = match get_attr_value(start, "name") {
            Some(n) => n,
            None => return Ok(None),
        };

        let qname = QName::new(target_ns, name);
        let mut base_type = TypeRef::string();
        let mut facets = RestrictionFacets::default();
        let mut enum_values = Vec::new();
        let mut documentation = None;
        let mut buf = Vec::new();

        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    depth += 1;
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "documentation" => {
                            let text = reader.read_text(e.name())?.to_string();
                            documentation = Some(text);
                            depth -= 1;
                        }
                        "restriction" => {
                            if let Some(base) = get_attr_value(e, "base") {
                                base_type = resolve_type_ref(&base, target_ns, prefixes);
                            }
                        }
                        "enumeration" => {
                            if let Some(val) = get_attr_value(e, "value") {
                                enum_values.push(EnumValue {
                                    name: sanitize_variant_name(&val),
                                    value: val.clone(),
                                    documentation: None,
                                });
                                facets.enumerations.push(val);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Empty(ref e) => {
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "restriction" => {
                            if let Some(base) = get_attr_value(e, "base") {
                                base_type = resolve_type_ref(&base, target_ns, prefixes);
                            }
                        }
                        "enumeration" => {
                            if let Some(val) = get_attr_value(e, "value") {
                                enum_values.push(EnumValue {
                                    name: sanitize_variant_name(&val),
                                    value: val.clone(),
                                    documentation: None,
                                });
                                facets.enumerations.push(val);
                            }
                        }
                        "pattern" => {
                            if let Some(val) = get_attr_value(e, "value") {
                                facets.patterns.push(val);
                            }
                        }
                        "minInclusive" => {
                            facets.min_inclusive = get_attr_value(e, "value");
                        }
                        "maxInclusive" => {
                            facets.max_inclusive = get_attr_value(e, "value");
                        }
                        "minExclusive" => {
                            facets.min_exclusive = get_attr_value(e, "value");
                        }
                        "maxExclusive" => {
                            facets.max_exclusive = get_attr_value(e, "value");
                        }
                        "minLength" => {
                            facets.min_length =
                                get_attr_value(e, "value").and_then(|v| v.parse().ok());
                        }
                        "maxLength" => {
                            facets.max_length =
                                get_attr_value(e, "value").and_then(|v| v.parse().ok());
                        }
                        "length" => {
                            facets.length = get_attr_value(e, "value").and_then(|v| v.parse().ok());
                        }
                        "totalDigits" => {
                            facets.total_digits =
                                get_attr_value(e, "value").and_then(|v| v.parse().ok());
                        }
                        "fractionDigits" => {
                            facets.fraction_digits =
                                get_attr_value(e, "value").and_then(|v| v.parse().ok());
                        }
                        "whiteSpace" => {
                            facets.white_space = get_attr_value(e, "value");
                        }
                        _ => {}
                    }
                }
                Event::End(_) => {
                    depth -= 1;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        if !enum_values.is_empty() {
            Ok(Some(TypeDef::Enum(EnumDef {
                qname,
                base_type,
                variants: enum_values,
                documentation,
            })))
        } else {
            Ok(Some(TypeDef::Simple(Box::new(SimpleTypeDef {
                qname,
                base_type,
                facets,
                documentation,
            }))))
        }
    }

    fn parse_global_element(
        &self,
        reader: &mut Reader<&[u8]>,
        start: &BytesStart,
        target_ns: Option<&str>,
        prefixes: &HashMap<String, String>,
        ir: &mut SchemaIR,
    ) -> Result<Option<ElementDef>, SchemaError> {
        let name = match get_attr_value(start, "name") {
            Some(n) => n,
            None => return Ok(None),
        };

        let qname = QName::new(target_ns, name.clone());
        let substitution_group = get_attr_value(start, "substitutionGroup")
            .map(|s| resolve_qname(&s, target_ns, prefixes));
        let nillable = get_attr_value(start, "nillable")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let mut type_ref = get_attr_value(start, "type")
            .map(|t| resolve_type_ref(&t, target_ns, prefixes))
            .unwrap_or(TypeRef::Primitive(PrimitiveType::AnyType));

        let mut documentation = None;
        let mut buf = Vec::new();

        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    depth += 1;
                    let local = strip_prefix(e.name().into_inner());

                    match local {
                        "documentation" => {
                            let text = reader.read_text(e.name())?.to_string();
                            documentation = Some(text);
                            depth -= 1;
                        }
                        "complexType" => {
                            let anon_name = format!("{}Type", name);
                            let anon_qname = QName::new(target_ns, anon_name);
                            if let Some(TypeDef::Struct(mut s)) =
                                self.parse_complex_type(reader, e, target_ns, prefixes)?
                            {
                                s.qname = anon_qname.clone();
                                ir.add_type(TypeDef::Struct(s));
                                type_ref = TypeRef::Named(anon_qname);
                            }
                            depth -= 1;
                        }
                        "simpleType" => {
                            let anon_name = format!("{}SimpleType", name);
                            let anon_qname = QName::new(target_ns, anon_name);
                            if let Some(type_def) =
                                self.parse_simple_type(reader, e, target_ns, prefixes)?
                            {
                                match type_def {
                                    TypeDef::Enum(mut ed) => {
                                        ed.qname = anon_qname.clone();
                                        ir.add_type(TypeDef::Enum(ed));
                                    }
                                    TypeDef::Simple(mut sd) => {
                                        sd.qname = anon_qname.clone();
                                        ir.add_type(TypeDef::Simple(sd));
                                    }
                                    _ => {}
                                }
                                type_ref = TypeRef::Named(anon_qname);
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                }
                Event::End(_) => {
                    depth -= 1;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(Some(ElementDef {
            qname,
            type_ref,
            substitution_group,
            nillable,
            documentation,
        }))
    }
}

// Helpers

fn strip_prefix(s: &str) -> &str {
    s.split_once(':').map(|(_, local)| local).unwrap_or(s)
}

fn get_attr_value(e: &BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if key == name || strip_prefix(key) == name {
            return Some(attr.value.as_ref().to_string());
        }
    }
    None
}

fn resolve_qname(name: &str, target_ns: Option<&str>, prefixes: &HashMap<String, String>) -> QName {
    if let Some((prefix, local)) = name.split_once(':') {
        let ns = prefixes.get(prefix).cloned();
        QName::new(ns, local)
    } else {
        QName::new(target_ns, name)
    }
}

fn resolve_type_ref(
    name: &str,
    target_ns: Option<&str>,
    prefixes: &HashMap<String, String>,
) -> TypeRef {
    if let Some(prim) = PrimitiveType::from_xsd_name(name) {
        return TypeRef::Primitive(prim);
    }

    if let Some((prefix, local)) = name.split_once(':') {
        if prefix == "xs" || prefix == "xsd" {
            if let Some(prim) = PrimitiveType::from_xsd_name(local) {
                return TypeRef::Primitive(prim);
            }
        }
        let ns = prefixes.get(prefix).cloned();
        TypeRef::Named(QName::new(ns, local))
    } else {
        TypeRef::Named(QName::new(target_ns, name))
    }
}

fn parse_element_field(
    e: &BytesStart,
    target_ns: Option<&str>,
    prefixes: &HashMap<String, String>,
    in_choice: bool,
) -> Option<FieldDef> {
    let name = get_attr_value(e, "name")
        .or_else(|| get_attr_value(e, "ref").map(|r| strip_prefix(&r).to_string()))?;

    let xml_name = get_attr_value(e, "name")
        .or_else(|| get_attr_value(e, "ref"))
        .unwrap_or_else(|| name.clone());

    let type_ref = get_attr_value(e, "type")
        .map(|t| resolve_type_ref(&t, target_ns, prefixes))
        .or_else(|| get_attr_value(e, "ref").map(|r| resolve_type_ref(&r, target_ns, prefixes)))
        .unwrap_or(TypeRef::Primitive(PrimitiveType::String));

    let min_occurs = if in_choice {
        0
    } else {
        get_attr_value(e, "minOccurs")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    };

    let max_occurs = match get_attr_value(e, "maxOccurs").as_deref() {
        Some("unbounded") => OccursLimit::Unbounded,
        Some(v) => OccursLimit::Count(v.parse().unwrap_or(1)),
        None => OccursLimit::Count(1),
    };

    let nillable = get_attr_value(e, "nillable")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let default_value = get_attr_value(e, "default");
    let fixed_value = get_attr_value(e, "fixed");

    Some(FieldDef {
        name: sanitize_field_name(&name),
        xml_name,
        namespace: target_ns.map(Into::into),
        kind: FieldKind::Element,
        type_ref,
        cardinality: Cardinality {
            min_occurs,
            max_occurs,
        },
        nillable,
        default_value,
        fixed_value,
        documentation: None,
        facets: None,
        is_cycle_cut: false,
    })
}

fn parse_attribute_field(
    e: &BytesStart,
    target_ns: Option<&str>,
    prefixes: &HashMap<String, String>,
) -> Option<FieldDef> {
    let name = get_attr_value(e, "name")
        .or_else(|| get_attr_value(e, "ref").map(|r| strip_prefix(&r).to_string()))?;

    let xml_name = get_attr_value(e, "name")
        .or_else(|| get_attr_value(e, "ref"))
        .unwrap_or_else(|| name.clone());

    let type_ref = get_attr_value(e, "type")
        .map(|t| resolve_type_ref(&t, target_ns, prefixes))
        .or_else(|| get_attr_value(e, "ref").map(|r| resolve_type_ref(&r, target_ns, prefixes)))
        .unwrap_or(TypeRef::Primitive(PrimitiveType::String));

    let is_required = get_attr_value(e, "use")
        .map(|u| u == "required")
        .unwrap_or(false);

    let cardinality = if is_required {
        Cardinality::required_one()
    } else {
        Cardinality::optional_one()
    };

    let default_value = get_attr_value(e, "default");
    let fixed_value = get_attr_value(e, "fixed");

    Some(FieldDef {
        name: sanitize_field_name(&name),
        xml_name,
        namespace: None, // Attributes are unqualified by default unless form="qualified"
        kind: FieldKind::Attribute,
        type_ref,
        cardinality,
        nillable: false,
        default_value,
        fixed_value,
        documentation: None,
        facets: None,
        is_cycle_cut: false,
    })
}

fn parse_any_field(_e: &BytesStart) -> FieldDef {
    FieldDef {
        name: "any".to_string(),
        xml_name: "*".to_string(),
        namespace: None,
        kind: FieldKind::Any,
        type_ref: TypeRef::Primitive(PrimitiveType::AnyType),
        cardinality: Cardinality::unbounded(0),
        nillable: false,
        default_value: None,
        fixed_value: None,
        documentation: None,
        facets: None,
        is_cycle_cut: false,
    }
}

fn parse_empty_global_element(
    e: &BytesStart,
    target_ns: Option<&str>,
    prefixes: &HashMap<String, String>,
) -> Option<ElementDef> {
    let name = get_attr_value(e, "name")?;
    let qname = QName::new(target_ns, name);
    let substitution_group =
        get_attr_value(e, "substitutionGroup").map(|s| resolve_qname(&s, target_ns, prefixes));
    let nillable = get_attr_value(e, "nillable")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let type_ref = get_attr_value(e, "type")
        .map(|t| resolve_type_ref(&t, target_ns, prefixes))
        .unwrap_or(TypeRef::Primitive(PrimitiveType::AnyType));

    Some(ElementDef {
        qname,
        type_ref,
        substitution_group,
        nillable,
        documentation: None,
    })
}

fn sanitize_field_name(name: &str) -> String {
    let s = heck::AsSnakeCase(name).to_string();
    if s.is_empty() {
        "field".to_string()
    } else if s.chars().next().unwrap().is_ascii_digit() {
        format!("_{}", s)
    } else {
        s
    }
}

fn sanitize_variant_name(name: &str) -> String {
    let s = heck::AsPascalCase(name).to_string();
    if s.is_empty() {
        "Variant".to_string()
    } else if s.chars().next().unwrap().is_ascii_digit() {
        format!("V{}", s)
    } else {
        s
    }
}

fn merge_ir(dest: &mut SchemaIR, src: SchemaIR) {
    for (k, v) in src.types {
        dest.types.insert(k, v);
    }
    for (k, v) in src.elements {
        dest.elements.insert(k, v);
    }
    for (k, v) in src.substitution_groups {
        dest.substitution_groups.entry(k).or_default().extend(v);
    }
}
