use std::fmt::Write as FmtWrite;

use heck::{AsPascalCase, AsSnakeCase};
use serde::{Deserialize, Serialize};

use crate::codegen::{sanitize_keyword, LanguageContext};
use crate::ir::{
    EnumDef, FieldDef, FieldKind, PrimitiveType, RestrictionFacets, SchemaIR, SimpleTypeDef,
    StructDef, TypeDef, TypeRef, UnionDef,
};

/// Options configuring Go 1.22+ code generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoOptions {
    /// Package name for generated Go code (default: "models")
    pub package_name: String,
    /// Emit encoding/xml tags (default: true)
    pub emit_xml_tags: bool,
    /// Emit custom UnmarshalXML/MarshalXML for xs:choice mutual exclusivity validation (default: true)
    pub validate_choice_exclusivity: bool,
    /// Emit Validate() error method for restriction facets (default: true)
    pub validate_facets: bool,
    /// Emit top-level type aliases for root elements (default: true)
    pub emit_root_aliases: bool,
}

impl Default for GoOptions {
    fn default() -> Self {
        Self {
            package_name: "models".to_string(),
            emit_xml_tags: true,
            validate_choice_exclusivity: true,
            validate_facets: true,
            emit_root_aliases: true,
        }
    }
}

/// Language context adapter for Go 1.22+.
pub struct GoLanguageContext;

impl LanguageContext for GoLanguageContext {
    fn target_language(&self) -> &'static str {
        "go"
    }

    fn map_primitive(&self, prim: PrimitiveType) -> &'static str {
        match prim {
            PrimitiveType::Boolean => "bool",
            PrimitiveType::Float => "float32",
            PrimitiveType::Double | PrimitiveType::Decimal => "float64",
            PrimitiveType::Byte => "int8",
            PrimitiveType::Short => "int16",
            PrimitiveType::Int => "int32",
            PrimitiveType::Integer
            | PrimitiveType::Long
            | PrimitiveType::PositiveInteger
            | PrimitiveType::NegativeInteger
            | PrimitiveType::NonPositiveInteger
            | PrimitiveType::NonNegativeInteger => "int64",
            PrimitiveType::UnsignedByte => "uint8",
            PrimitiveType::UnsignedShort => "uint16",
            PrimitiveType::UnsignedInt => "uint32",
            PrimitiveType::UnsignedLong => "uint64",
            PrimitiveType::String
            | PrimitiveType::NormalizedString
            | PrimitiveType::Token
            | PrimitiveType::Name
            | PrimitiveType::NCName
            | PrimitiveType::QName
            | PrimitiveType::Language
            | PrimitiveType::NMTOKEN
            | PrimitiveType::NMTOKENS
            | PrimitiveType::AnyUri
            | PrimitiveType::Id
            | PrimitiveType::IdRef
            | PrimitiveType::IdRefs
            | PrimitiveType::Entity
            | PrimitiveType::Entities
            | PrimitiveType::GYear
            | PrimitiveType::GYearMonth
            | PrimitiveType::GMonth
            | PrimitiveType::GMonthDay
            | PrimitiveType::GDay
            | PrimitiveType::Duration => "string",
            PrimitiveType::Date | PrimitiveType::Time | PrimitiveType::DateTime => "time.Time",
            PrimitiveType::Base64Binary | PrimitiveType::HexBinary => "[]byte",
            PrimitiveType::AnyType | PrimitiveType::AnySimpleType => "any",
        }
    }

    fn map_type_ref(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => self.map_primitive(*prim).to_string(),
            TypeRef::Named(qname) => to_go_type_name(&qname.local),
            TypeRef::Boxed(inner) => format!("*{}", self.map_type_ref(inner)),
            TypeRef::List(inner) => format!("[]{}", self.map_type_ref(inner)),
        }
    }
}

/// Normalizes common Go initialisms (e.g. `Id` -> `ID`, `Url` -> `URL`).
fn normalize_go_initialisms(s: &str) -> String {
    if s == "Id" {
        return "ID".to_string();
    }
    if s == "Url" {
        return "URL".to_string();
    }
    if s == "Xml" {
        return "XML".to_string();
    }
    if s == "Uri" {
        return "URI".to_string();
    }
    if s == "Uuid" {
        return "UUID".to_string();
    }

    let mut res = s.to_string();
    if res.ends_with("Id") {
        let len = res.len();
        res.replace_range(len - 2.., "ID");
    } else if res.ends_with("Url") {
        let len = res.len();
        res.replace_range(len - 3.., "URL");
    } else if res.ends_with("Xml") {
        let len = res.len();
        res.replace_range(len - 3.., "XML");
    } else if res.ends_with("Uri") {
        let len = res.len();
        res.replace_range(len - 3.., "URI");
    }

    if res.starts_with("Xml") {
        res.replace_range(..3, "XML");
    }
    res
}

/// Converts a raw identifier into an exported PascalCase Go type name.
pub fn to_go_type_name(raw: &str) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let normalized = normalize_go_initialisms(&pascal);
    let safe = if normalized.is_empty() {
        "Type".to_string()
    } else if normalized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("Type{}", normalized)
    } else {
        normalized
    };
    sanitize_keyword(&safe, "go")
}

/// Converts a raw identifier into an exported PascalCase Go struct field name.
pub fn to_go_field_name(raw: &str) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let normalized = normalize_go_initialisms(&pascal);
    let safe = if normalized.is_empty() {
        "Field".to_string()
    } else if normalized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("Field{}", normalized)
    } else {
        normalized
    };
    sanitize_keyword(&safe, "go")
}

/// Converts an enumeration variant into a typed Go constant identifier (`EnumNameVariant`).
pub fn to_go_constant_name(type_name: &str, raw: &str) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let normalized = normalize_go_initialisms(&pascal);
    let safe_var = if normalized.is_empty() {
        "Value".to_string()
    } else if normalized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("V{}", normalized)
    } else {
        normalized
    };
    format!("{}{}", type_name, safe_var)
}

/// Sanitizes a string into a valid Go package name (`crm`, `models`, etc.).
pub fn to_go_package_name(raw: &str) -> String {
    let s = AsSnakeCase(raw).to_string().replace('_', "");
    let safe = if s.is_empty() {
        "models".to_string()
    } else if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("pkg{}", s)
    } else {
        s
    };
    sanitize_keyword(&safe, "go")
}

/// Modern Go 1.22+ Code Generator.
pub struct GoCodegen {
    options: GoOptions,
    context: GoLanguageContext,
}

impl GoCodegen {
    pub fn new(options: GoOptions) -> Self {
        Self {
            options,
            context: GoLanguageContext,
        }
    }

    /// Generate complete Go module content.
    pub fn generate_module(&self, ir: &SchemaIR) -> String {
        let mut body = String::new();
        let mut has_time = false;
        let mut has_fmt = false;
        let mut has_io = false;
        let mut has_xml = self.options.emit_xml_tags;

        // Check types for time.Time
        for type_def in ir.types.values() {
            match type_def {
                TypeDef::Struct(s) => {
                    for f in &s.fields {
                        if self.references_time(&f.type_ref) {
                            has_time = true;
                        }
                    }
                    if self.options.validate_facets
                        && s.fields.iter().any(|f| {
                            f.facets
                                .as_ref()
                                .map(|fac| !fac.is_empty())
                                .unwrap_or(false)
                        })
                    {
                        has_fmt = true;
                    }
                }
                TypeDef::Union(_) => {
                    if self.options.validate_choice_exclusivity {
                        has_fmt = true;
                        has_io = true;
                        has_xml = true;
                    }
                }
                TypeDef::Simple(st) => {
                    if self.references_time(&st.base_type) {
                        has_time = true;
                    }
                }
                TypeDef::Enum(_) => {}
            }
        }

        // Generate types
        self.emit_types(&mut body, ir);
        self.emit_root_aliases(&mut body, ir);

        // Assemble final output with package and imports
        let mut out = String::new();
        writeln!(
            out,
            "// Code generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML). DO NOT EDIT."
        )
        .unwrap();

        let pkg = to_go_package_name(&self.options.package_name);
        writeln!(out, "package {}\n", pkg).unwrap();

        let mut imports = Vec::new();
        if has_xml {
            imports.push("\"encoding/xml\"");
        }
        if has_fmt {
            imports.push("\"fmt\"");
        }
        if has_io {
            imports.push("\"io\"");
        }
        if has_time {
            imports.push("\"time\"");
        }

        if !imports.is_empty() {
            writeln!(out, "import (").unwrap();
            for imp in imports {
                writeln!(out, "    {}", imp).unwrap();
            }
            writeln!(out, ")\n").unwrap();
        }

        out.push_str(&body);
        out
    }

    /// Generate bundle of files (source file named after base_name).
    pub fn generate_files(&self, ir: &SchemaIR, base_name: &str) -> Vec<(String, String)> {
        let filename = if base_name.is_empty() {
            "models.go".to_string()
        } else {
            format!("{}.go", AsSnakeCase(base_name))
        };

        vec![(filename, self.generate_module(ir))]
    }

    fn references_time(&self, type_ref: &TypeRef) -> bool {
        match type_ref {
            TypeRef::Primitive(
                PrimitiveType::Date | PrimitiveType::Time | PrimitiveType::DateTime,
            ) => true,
            TypeRef::List(inner) | TypeRef::Boxed(inner) => self.references_time(inner),
            _ => false,
        }
    }

    fn emit_types(&self, out: &mut String, ir: &SchemaIR) {
        // Emit simple types
        for type_def in ir.types.values() {
            if let TypeDef::Simple(simple) = type_def {
                self.emit_simple_type(out, simple);
            }
        }

        // Emit enums
        for type_def in ir.types.values() {
            if let TypeDef::Enum(enum_def) = type_def {
                self.emit_enum(out, enum_def);
            }
        }

        // Emit choices (unions)
        for type_def in ir.types.values() {
            if let TypeDef::Union(u) = type_def {
                self.emit_union(out, u);
            }
        }

        // Emit structs
        for type_def in ir.types.values() {
            if let TypeDef::Struct(s) = type_def {
                self.emit_struct(out, s, ir);
            }
        }
    }

    fn emit_simple_type(&self, out: &mut String, simple: &SimpleTypeDef) {
        if let Some(ref doc) = simple.documentation {
            for line in doc.lines() {
                writeln!(out, "// {}", line).unwrap();
            }
        }

        let type_name = to_go_type_name(&simple.qname.local);
        let base_type = self.context.map_type_ref(&simple.base_type);
        writeln!(out, "type {} {}\n", type_name, base_type).unwrap();
    }

    fn emit_enum(&self, out: &mut String, enum_def: &EnumDef) {
        let enum_name = to_go_type_name(&enum_def.qname.local);
        if let Some(ref doc) = enum_def.documentation {
            for line in doc.lines() {
                writeln!(out, "// {}", line).unwrap();
            }
        }

        writeln!(out, "type {} string\n", enum_name).unwrap();

        writeln!(out, "const (").unwrap();
        for variant in &enum_def.variants {
            if let Some(ref doc) = variant.documentation {
                writeln!(out, "    // {}", doc).unwrap();
            }
            let const_name = to_go_constant_name(&enum_name, &variant.name);
            writeln!(
                out,
                "    {} {} = \"{}\"",
                const_name, enum_name, variant.value
            )
            .unwrap();
        }
        writeln!(out, ")\n").unwrap();

        // IsValid() bool method
        writeln!(out, "func (e {}) IsValid() bool {{", enum_name).unwrap();
        writeln!(out, "    switch e {{").unwrap();
        let const_names: Vec<String> = enum_def
            .variants
            .iter()
            .map(|v| to_go_constant_name(&enum_name, &v.name))
            .collect();
        writeln!(out, "    case {}:", const_names.join(", ")).unwrap();
        writeln!(out, "        return true").unwrap();
        writeln!(out, "    default:").unwrap();
        writeln!(out, "        return false").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}\n").unwrap();
    }

    fn emit_union(&self, out: &mut String, u: &UnionDef) {
        let choice_name = to_go_type_name(&u.qname.local);
        if let Some(ref doc) = u.documentation {
            for line in doc.lines() {
                writeln!(out, "// {}", line).unwrap();
            }
        }

        // Choice container struct
        writeln!(out, "type {} struct {{", choice_name).unwrap();
        for branch in &u.branches {
            if let Some(ref doc) = branch.documentation {
                writeln!(out, "    // {}", doc).unwrap();
            }
            let field_name = to_go_field_name(&branch.variant_name);
            let mapped_type = self.context.map_type_ref(&branch.type_ref);
            let xml_tag = if self.options.emit_xml_tags {
                format!(" `xml:\"{},omitempty\"`", branch.xml_name)
            } else {
                String::new()
            };
            writeln!(out, "    {} *{}{}", field_name, mapped_type, xml_tag).unwrap();
        }
        writeln!(out, "}}\n").unwrap();

        // Selected() string helper
        writeln!(out, "func (c {}) Selected() string {{", choice_name).unwrap();
        for branch in &u.branches {
            let field_name = to_go_field_name(&branch.variant_name);
            writeln!(out, "    if c.{} != nil {{", field_name).unwrap();
            writeln!(out, "        return \"{}\"", branch.xml_name).unwrap();
            writeln!(out, "    }}").unwrap();
        }
        writeln!(out, "    return \"\"").unwrap();
        writeln!(out, "}}\n").unwrap();

        // Validate() error method
        writeln!(out, "func (c {}) Validate() error {{", choice_name).unwrap();
        writeln!(out, "    count := 0").unwrap();
        for branch in &u.branches {
            let field_name = to_go_field_name(&branch.variant_name);
            writeln!(out, "    if c.{} != nil {{ count++ }}", field_name).unwrap();
        }
        writeln!(out, "    if count > 1 {{").unwrap();
        writeln!(
            out,
            "        return fmt.Errorf(\"choice {} mutual exclusivity violation: multiple branches populated (%d)\", count)",
            choice_name
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "    return nil").unwrap();
        writeln!(out, "}}\n").unwrap();

        if self.options.validate_choice_exclusivity {
            // UnmarshalXML receiver method
            writeln!(
                out,
                "func (c *{}) UnmarshalXML(d *xml.Decoder, start xml.StartElement) error {{",
                choice_name
            )
            .unwrap();
            writeln!(out, "    var raw {}", choice_name).unwrap();
            writeln!(out, "    count := 0\n").unwrap();
            writeln!(out, "    for {{").unwrap();
            writeln!(out, "        tok, err := d.Token()").unwrap();
            writeln!(out, "        if err != nil {{").unwrap();
            writeln!(out, "            if err == io.EOF {{").unwrap();
            writeln!(out, "                break").unwrap();
            writeln!(out, "            }}").unwrap();
            writeln!(out, "            return err").unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "        switch t := tok.(type) {{").unwrap();
            writeln!(out, "        case xml.StartElement:").unwrap();
            writeln!(out, "            switch t.Name.Local {{").unwrap();

            for branch in &u.branches {
                let field_name = to_go_field_name(&branch.variant_name);
                let mapped_type = self.context.map_type_ref(&branch.type_ref);
                writeln!(out, "            case \"{}\":", branch.xml_name).unwrap();
                writeln!(out, "                var v {}", mapped_type).unwrap();
                writeln!(
                    out,
                    "                if err := d.DecodeElement(&v, &t); err != nil {{"
                )
                .unwrap();
                writeln!(out, "                    return err").unwrap();
                writeln!(out, "                }}").unwrap();
                writeln!(out, "                raw.{} = &v", field_name).unwrap();
                writeln!(out, "                count++").unwrap();
            }

            writeln!(out, "            default:").unwrap();
            writeln!(out, "                if err := d.Skip(); err != nil {{").unwrap();
            writeln!(out, "                    return err").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
            writeln!(out, "        case xml.EndElement:").unwrap();
            writeln!(out, "            if t == start.End() {{").unwrap();
            writeln!(out, "                goto validation").unwrap();
            writeln!(out, "            }}").unwrap();
            writeln!(out, "        }}").unwrap();
            writeln!(out, "    }}\n").unwrap();
            writeln!(out, "validation:").unwrap();
            writeln!(out, "    if count > 1 {{").unwrap();
            writeln!(
                out,
                "        return fmt.Errorf(\"choice {} mutual exclusivity violation: multiple branches populated (%d)\", count)",
                choice_name
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(out, "    *c = raw").unwrap();
            writeln!(out, "    return nil").unwrap();
            writeln!(out, "}}\n").unwrap();

            // MarshalXML receiver method
            writeln!(
                out,
                "func (c {}) MarshalXML(e *xml.Encoder, start xml.StartElement) error {{",
                choice_name
            )
            .unwrap();
            writeln!(out, "    if err := c.Validate(); err != nil {{").unwrap();
            writeln!(out, "        return err").unwrap();
            writeln!(out, "    }}").unwrap();
            writeln!(out, "    type Alias {}", choice_name).unwrap();
            writeln!(out, "    return e.EncodeElement(Alias(c), start)").unwrap();
            writeln!(out, "}}\n").unwrap();
        }
    }

    fn emit_struct(&self, out: &mut String, s: &StructDef, ir: &SchemaIR) {
        let struct_name = to_go_type_name(&s.qname.local);
        if let Some(ref doc) = s.documentation {
            for line in doc.lines() {
                writeln!(out, "// {}", line).unwrap();
            }
        }

        writeln!(out, "type {} struct {{", struct_name).unwrap();

        // Emit XMLName if xml tags enabled
        if self.options.emit_xml_tags {
            writeln!(out, "    XMLName xml.Name").unwrap();
        }

        // Struct composition / inheritance if base struct exists
        let mut simple_content_base: Option<String> = None;
        if let Some(ref base_qname) = s.base_type {
            if matches!(ir.types.get(base_qname), Some(TypeDef::Struct(_))) {
                let base_name = to_go_type_name(&base_qname.local);
                writeln!(out, "    {}", base_name).unwrap();
            } else if let Some(prim) = PrimitiveType::from_xsd_name(&base_qname.local) {
                simple_content_base = Some(self.context.map_primitive(prim).to_string());
            } else if let Some(TypeDef::Simple(st)) = ir.types.get(base_qname) {
                simple_content_base = Some(to_go_type_name(&st.qname.local));
            }
        }

        if let Some(base_type_str) = simple_content_base {
            let has_value_field = s
                .fields
                .iter()
                .any(|f| f.name == "value" || f.kind == FieldKind::Text);
            if !has_value_field {
                let tag = if self.options.emit_xml_tags {
                    " `xml:\",chardata\"`"
                } else {
                    ""
                };
                writeln!(out, "    Value {}{}", base_type_str, tag).unwrap();
            }
        }

        // Fields
        for f in &s.fields {
            if let Some(ref doc) = f.documentation {
                writeln!(out, "    // {}", doc).unwrap();
            }

            let field_name = to_go_field_name(&f.name);
            let field_type = self.resolve_field_type(f);
            let tag = self.build_field_xml_tag(f);
            writeln!(out, "    {} {}{}", field_name, field_type, tag).unwrap();
        }

        writeln!(out, "}}\n").unwrap();

        if self.options.validate_facets {
            self.emit_struct_validator(out, s);
        }
    }

    fn resolve_field_type(&self, f: &FieldDef) -> String {
        let base_type = self.context.map_type_ref(&f.type_ref);

        if f.cardinality.is_list() {
            if f.is_cycle_cut {
                format!("[]*{}", base_type)
            } else {
                format!("[]{}", base_type)
            }
        } else if f.is_cycle_cut || f.cardinality.is_optional() || f.nillable {
            format!("*{}", base_type)
        } else {
            base_type
        }
    }

    fn build_field_xml_tag(&self, f: &FieldDef) -> String {
        if !self.options.emit_xml_tags {
            return String::new();
        }

        let is_opt = f.cardinality.is_optional() || f.nillable;
        match f.kind {
            FieldKind::Attribute => {
                if is_opt {
                    format!(" `xml:\"{},attr,omitempty\"`", f.xml_name)
                } else {
                    format!(" `xml:\"{},attr\"`", f.xml_name)
                }
            }
            FieldKind::Text => " `xml:\",chardata\"`".to_string(),
            FieldKind::Any => " `xml:\",any\"`".to_string(),
            FieldKind::AnyAttribute => " `xml:\",any,attr\"`".to_string(),
            FieldKind::Element => {
                if is_opt {
                    format!(" `xml:\"{},omitempty\"`", f.xml_name)
                } else {
                    format!(" `xml:\"{}\"`", f.xml_name)
                }
            }
        }
    }

    fn emit_struct_validator(&self, out: &mut String, s: &StructDef) {
        let struct_name = to_go_type_name(&s.qname.local);
        writeln!(out, "func (s {}) Validate() error {{", struct_name).unwrap();

        let mut has_checks = false;
        for f in &s.fields {
            let field_name = to_go_field_name(&f.name);
            let is_opt = f.cardinality.is_optional() || f.nillable;

            if let Some(ref facets) = f.facets {
                if is_opt {
                    writeln!(out, "    if s.{} != nil {{", field_name).unwrap();
                    self.emit_facet_checks(
                        out,
                        facets,
                        &format!("(*s.{})", field_name),
                        "        ",
                    );
                    writeln!(out, "    }}").unwrap();
                } else {
                    self.emit_facet_checks(out, facets, &format!("s.{}", field_name), "    ");
                }
                has_checks = true;
            }
        }

        let _ = has_checks;
        writeln!(out, "    return nil").unwrap();
        writeln!(out, "}}\n").unwrap();
    }

    fn emit_facet_checks(
        &self,
        out: &mut String,
        facets: &RestrictionFacets,
        target: &str,
        indent: &str,
    ) {
        if let Some(min_len) = facets.min_length {
            writeln!(
                out,
                "{}if len({}) < {} {{ return fmt.Errorf(\"field violates minLength constraint ({})\") }}",
                indent, target, min_len, min_len
            )
            .unwrap();
        }
        if let Some(max_len) = facets.max_length {
            writeln!(
                out,
                "{}if len({}) > {} {{ return fmt.Errorf(\"field violates maxLength constraint ({})\") }}",
                indent, target, max_len, max_len
            )
            .unwrap();
        }
        if let Some(len) = facets.length {
            writeln!(
                out,
                "{}if len({}) != {} {{ return fmt.Errorf(\"field violates length constraint ({})\") }}",
                indent, target, len, len
            )
            .unwrap();
        }
        if let Some(ref min_inc) = facets.min_inclusive {
            writeln!(
                out,
                "{}if {} < {} {{ return fmt.Errorf(\"field violates minInclusive constraint ({})\") }}",
                indent, target, min_inc, min_inc
            )
            .unwrap();
        }
        if let Some(ref max_inc) = facets.max_inclusive {
            writeln!(
                out,
                "{}if {} > {} {{ return fmt.Errorf(\"field violates maxInclusive constraint ({})\") }}",
                indent, target, max_inc, max_inc
            )
            .unwrap();
        }
    }

    fn emit_root_aliases(&self, out: &mut String, ir: &SchemaIR) {
        if !self.options.emit_root_aliases || ir.elements.is_empty() {
            return;
        }

        writeln!(out, "// Root XML Element Type Aliases").unwrap();
        for elem in ir.elements.values() {
            let elem_alias = to_go_type_name(&elem.qname.local);
            let target_type = self.context.map_type_ref(&elem.type_ref);
            if elem_alias != target_type {
                if let Some(ref doc) = elem.documentation {
                    writeln!(out, "// {}", doc).unwrap();
                }
                writeln!(out, "type {} = {}\n", elem_alias, target_type).unwrap();
            }
        }
    }
}
