//! Modern C# 12 / .NET 8+ Code Generator for PolyXML-IR.
//!
//! Emits idiomatic C# 12 records with primary constructors, standard System.Xml.Serialization
//! attributes, polymorphic xs:choice abstract records, and IValidatableObject facet boundary checks.

use std::fmt::Write as FmtWrite;

use heck::{AsLowerCamelCase, AsPascalCase};
use serde::{Deserialize, Serialize};

use crate::codegen::{sanitize_keyword, LanguageContext};
use crate::ir::{
    EnumDef, FieldDef, FieldKind, PrimitiveType, RestrictionFacets, SchemaIR, SimpleTypeDef,
    StructDef, TypeDef, TypeRef, UnionDef,
};

/// Record emission kind: class vs struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CSharpRecordKind {
    /// Emit `public sealed record` (reference type with value semantics)
    #[default]
    Class,
    /// Emit `public readonly record struct` (value type)
    Struct,
}

/// Options configuring C# 12 / .NET 8+ code generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CSharpOptions {
    /// Namespace declaration (e.g. "Crm.Models", "Generated")
    pub namespace: String,
    /// Emit System.Xml.Serialization attributes ([XmlElement], [XmlAttribute], etc.)
    pub emit_xml_attributes: bool,
    /// Emit IValidatableObject and restriction facet validation logic
    pub emit_validation: bool,
    /// Record emission kind (record class vs record struct)
    pub record_kind: CSharpRecordKind,
    /// Use modern C# 10+ file-scoped namespaces (`namespace Foo;`)
    pub use_file_scoped_namespaces: bool,
    /// Emit root element wrapper records or aliases
    pub emit_root_records: bool,
}

impl Default for CSharpOptions {
    fn default() -> Self {
        Self {
            namespace: "Generated".to_string(),
            emit_xml_attributes: true,
            emit_validation: true,
            record_kind: CSharpRecordKind::Class,
            use_file_scoped_namespaces: true,
            emit_root_records: true,
        }
    }
}

/// Sanitizes an identifier into a PascalCase C# type name.
pub fn to_csharp_type_name(raw: &str) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let safe = if pascal.is_empty() {
        "Type".to_string()
    } else if pascal.starts_with(|c: char| c.is_ascii_digit()) {
        format!("Type{}", pascal)
    } else {
        pascal
    };
    sanitize_keyword(&safe, "csharp")
}

/// Sanitizes an identifier into a PascalCase C# property name.
/// If the property name matches the enclosing type name, suffixes "Value" to prevent CS0542.
pub fn to_csharp_property_name(raw: &str, enclosing_type: Option<&str>) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let safe = if pascal.is_empty() {
        "Property".to_string()
    } else if pascal.starts_with(|c: char| c.is_ascii_digit()) {
        format!("Prop{}", pascal)
    } else {
        pascal
    };

    let sanitized = sanitize_keyword(&safe, "csharp");
    if let Some(enclosing) = enclosing_type {
        if sanitized == enclosing || sanitized.trim_start_matches('@') == enclosing {
            return format!("{}Value", sanitized);
        }
    }
    sanitized
}

/// Sanitizes an identifier into a camelCase C# parameter name.
pub fn to_csharp_param_name(raw: &str) -> String {
    let camel = AsLowerCamelCase(raw).to_string();
    let safe = if camel.is_empty() {
        "item".to_string()
    } else if camel.starts_with(|c: char| c.is_ascii_digit()) {
        format!("p{}", camel)
    } else {
        camel
    };
    sanitize_keyword(&safe, "csharp")
}

/// Sanitizes an identifier into a PascalCase C# enum variant.
pub fn to_csharp_variant_name(raw: &str) -> String {
    let pascal = AsPascalCase(raw).to_string();
    let safe = if pascal.is_empty() {
        "Value".to_string()
    } else if pascal.starts_with(|c: char| c.is_ascii_digit()) {
        format!("V{}", pascal)
    } else {
        pascal
    };
    sanitize_keyword(&safe, "csharp")
}

/// Sanitizes a namespace into dotted PascalCase segments.
pub fn to_csharp_namespace(raw: &str) -> String {
    let segments: Vec<String> = raw
        .split(['.', '/', ':'])
        .filter(|s| !s.is_empty())
        .map(to_csharp_type_name)
        .collect();

    if segments.is_empty() {
        "Generated".to_string()
    } else {
        segments.join(".")
    }
}

/// Language context adapter for C# 12 / .NET 8+.
pub struct CSharpLanguageContext;

impl LanguageContext for CSharpLanguageContext {
    fn target_language(&self) -> &'static str {
        "csharp"
    }

    fn map_primitive(&self, prim: PrimitiveType) -> &'static str {
        match prim {
            PrimitiveType::String
            | PrimitiveType::NormalizedString
            | PrimitiveType::Token
            | PrimitiveType::Language
            | PrimitiveType::Name
            | PrimitiveType::NCName
            | PrimitiveType::NMTOKEN
            | PrimitiveType::NMTOKENS
            | PrimitiveType::Id
            | PrimitiveType::IdRef
            | PrimitiveType::IdRefs
            | PrimitiveType::Entity
            | PrimitiveType::Entities
            | PrimitiveType::AnyUri
            | PrimitiveType::QName
            | PrimitiveType::GYear
            | PrimitiveType::GYearMonth
            | PrimitiveType::GMonth
            | PrimitiveType::GMonthDay
            | PrimitiveType::GDay => "string",
            PrimitiveType::Boolean => "bool",
            PrimitiveType::Byte => "sbyte",
            PrimitiveType::UnsignedByte => "byte",
            PrimitiveType::Short => "short",
            PrimitiveType::UnsignedShort => "ushort",
            PrimitiveType::Int => "int",
            PrimitiveType::UnsignedInt => "uint",
            PrimitiveType::Long
            | PrimitiveType::Integer
            | PrimitiveType::NonPositiveInteger
            | PrimitiveType::NegativeInteger
            | PrimitiveType::NonNegativeInteger
            | PrimitiveType::PositiveInteger => "long",
            PrimitiveType::UnsignedLong => "ulong",
            PrimitiveType::Float => "float",
            PrimitiveType::Double => "double",
            PrimitiveType::Decimal => "decimal",
            PrimitiveType::DateTime => "DateTimeOffset",
            PrimitiveType::Date => "DateOnly",
            PrimitiveType::Time => "TimeOnly",
            PrimitiveType::Duration => "TimeSpan",
            PrimitiveType::Base64Binary | PrimitiveType::HexBinary => "byte[]",
            PrimitiveType::AnyType | PrimitiveType::AnySimpleType => "object",
        }
    }

    fn map_type_ref(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => self.map_primitive(*prim).to_string(),
            TypeRef::Named(qname) => to_csharp_type_name(&qname.local),
            TypeRef::Boxed(inner) => self.map_type_ref(inner),
            TypeRef::List(inner) => format!("List<{}>", self.map_type_ref(inner)),
        }
    }
}

/// Modern C# 12 / .NET 8+ Code Generator.
pub struct CSharpCodegen {
    options: CSharpOptions,
    context: CSharpLanguageContext,
}

impl CSharpCodegen {
    /// Creates a new C# code generator with the given configuration options.
    pub fn new(options: CSharpOptions) -> Self {
        Self {
            options,
            context: CSharpLanguageContext,
        }
    }

    /// Emits all types in the given SchemaIR as a single C# compilation unit.
    pub fn generate_module(&self, ir: &SchemaIR) -> String {
        let mut out = String::new();

        writeln!(out, "// <auto-generated/>").unwrap();
        writeln!(out, "#nullable enable\n").unwrap();
        writeln!(out, "using System;").unwrap();
        writeln!(out, "using System.Collections.Generic;").unwrap();
        if self.options.emit_validation {
            writeln!(out, "using System.ComponentModel.DataAnnotations;").unwrap();
            writeln!(out, "using System.Text.RegularExpressions;").unwrap();
        }
        if self.options.emit_xml_attributes {
            writeln!(out, "using System.Xml.Serialization;").unwrap();
        }
        writeln!(out).unwrap();

        let ns = to_csharp_namespace(&self.options.namespace);
        if self.options.use_file_scoped_namespaces {
            writeln!(out, "namespace {};\n", ns).unwrap();
        } else {
            writeln!(out, "namespace {}\n{{\n", ns).unwrap();
        }

        let indent = if self.options.use_file_scoped_namespaces {
            ""
        } else {
            "    "
        };

        // Emit SimpleTypes / Enums
        for def in ir.types.values() {
            if let TypeDef::Enum(e) = def {
                self.emit_enum(&mut out, e, indent);
            } else if let TypeDef::Simple(s) = def {
                self.emit_simple(&mut out, s, indent);
            }
        }

        // Emit Unions (Choices)
        for def in ir.types.values() {
            if let TypeDef::Union(u) = def {
                self.emit_union(&mut out, u, indent);
            }
        }

        // Emit Structs (ComplexTypes)
        for def in ir.types.values() {
            if let TypeDef::Struct(s) = def {
                self.emit_struct(&mut out, s, ir, indent);
            }
        }

        // Emit Root Element Records if requested
        if self.options.emit_root_records {
            self.emit_root_elements(&mut out, ir, indent);
        }

        if !self.options.use_file_scoped_namespaces {
            writeln!(out, "}}").unwrap();
        }

        out
    }

    /// Emits generated C# files suitable for multi-file project outputs.
    pub fn generate_files(&self, ir: &SchemaIR, base_name: &str) -> Vec<(String, String)> {
        let code = self.generate_module(ir);
        let filename = format!("{}.cs", to_csharp_type_name(base_name));
        vec![(filename, code)]
    }

    fn emit_enum(&self, out: &mut String, e: &EnumDef, indent: &str) {
        let enum_name = to_csharp_type_name(&e.qname.local);
        if let Some(ref doc) = e.documentation {
            self.emit_docstring(out, doc, indent);
        }

        writeln!(out, "{}public enum {}", indent, enum_name).unwrap();
        writeln!(out, "{}{{", indent).unwrap();

        for variant in &e.variants {
            let variant_name = to_csharp_variant_name(&variant.name);
            if let Some(ref doc) = variant.documentation {
                self.emit_docstring(out, doc, &format!("{}    ", indent));
            }
            if self.options.emit_xml_attributes {
                writeln!(out, "{}    [XmlEnum(\"{}\")]", indent, variant.value).unwrap();
            }
            writeln!(out, "{}    {},", indent, variant_name).unwrap();
        }

        writeln!(out, "{}}}\n", indent).unwrap();

        // Emit helper extension methods
        writeln!(out, "{}public static class {}Extensions", indent, enum_name).unwrap();
        writeln!(out, "{}{{", indent).unwrap();

        // IsValid extension
        writeln!(
            out,
            "{}    public static bool IsValid(this {} value) => value switch",
            indent, enum_name
        )
        .unwrap();
        writeln!(out, "{}    {{", indent).unwrap();
        for variant in &e.variants {
            let variant_name = to_csharp_variant_name(&variant.name);
            writeln!(
                out,
                "{}        {}.{} => true,",
                indent, enum_name, variant_name
            )
            .unwrap();
        }
        writeln!(out, "{}        _ => false", indent).unwrap();
        writeln!(out, "{}    }};\n", indent).unwrap();

        // ToXmlValue extension
        writeln!(
            out,
            "{}    public static string ToXmlValue(this {} value) => value switch",
            indent, enum_name
        )
        .unwrap();
        writeln!(out, "{}    {{", indent).unwrap();
        for variant in &e.variants {
            let variant_name = to_csharp_variant_name(&variant.name);
            writeln!(
                out,
                "{}        {}.{} => \"{}\",",
                indent, enum_name, variant_name, variant.value
            )
            .unwrap();
        }
        writeln!(
            out,
            "{}        _ => throw new ArgumentOutOfRangeException(nameof(value), value, null)",
            indent
        )
        .unwrap();
        writeln!(out, "{}    }};", indent).unwrap();

        writeln!(out, "{}}}\n", indent).unwrap();
    }

    fn emit_simple(&self, out: &mut String, s: &SimpleTypeDef, indent: &str) {
        if !s.facets.is_empty() {
            let type_name = to_csharp_type_name(&s.qname.local);
            let base_type = self.context.map_type_ref(&s.base_type);

            if let Some(ref doc) = s.documentation {
                self.emit_docstring(out, doc, indent);
            }

            writeln!(
                out,
                "{}public sealed record {}([property: XmlText] {} Value) : IValidatableObject",
                indent, type_name, base_type
            )
            .unwrap();
            writeln!(out, "{}{{", indent).unwrap();
            writeln!(
                out,
                "{}    public {}() : this(default({})!) {{ }}",
                indent, type_name, base_type
            )
            .unwrap();
            writeln!(out).unwrap();

            // Validation
            writeln!(
                out,
                "{}    public IEnumerable<ValidationResult> Validate(ValidationContext validationContext)",
                indent
            )
            .unwrap();
            writeln!(out, "{}    {{", indent).unwrap();
            self.emit_facet_checks(out, &s.facets, "Value", &format!("{}        ", indent));
            writeln!(out, "{}        yield break;", indent).unwrap();
            writeln!(out, "{}    }}", indent).unwrap();
            writeln!(out, "{}}}\n", indent).unwrap();
        }
    }

    fn emit_union(&self, out: &mut String, u: &UnionDef, indent: &str) {
        let choice_name = to_csharp_type_name(&u.qname.local);
        if let Some(ref doc) = u.documentation {
            self.emit_docstring(out, doc, indent);
        }

        // XmlInclude attributes for polymorphism
        if self.options.emit_xml_attributes {
            for branch in &u.branches {
                let variant_name = to_csharp_type_name(&branch.variant_name);
                writeln!(
                    out,
                    "{}[XmlInclude(typeof({}.{}))]",
                    indent, choice_name, variant_name
                )
                .unwrap();
            }
        }

        writeln!(out, "{}public abstract record {}", indent, choice_name).unwrap();
        writeln!(out, "{}{{", indent).unwrap();

        for branch in &u.branches {
            let variant_name = to_csharp_type_name(&branch.variant_name);
            let branch_type = self.context.map_type_ref(&branch.type_ref);

            if let Some(ref doc) = branch.documentation {
                self.emit_docstring(out, doc, &format!("{}    ", indent));
            }

            let xml_attr = if self.options.emit_xml_attributes {
                format!("[property: XmlElement(\"{}\")] ", branch.xml_name)
            } else {
                String::new()
            };

            writeln!(
                out,
                "{}    public sealed record {}({}{} Value) : {}",
                indent, variant_name, xml_attr, branch_type, choice_name
            )
            .unwrap();
            writeln!(out, "{}    {{", indent).unwrap();
            writeln!(
                out,
                "{}        public {}() : this(default({})!) {{ }}",
                indent, variant_name, branch_type
            )
            .unwrap();
            writeln!(out, "{}    }}", indent).unwrap();
            writeln!(out).unwrap();
        }

        writeln!(out, "{}}}\n", indent).unwrap();
    }

    fn emit_struct(&self, out: &mut String, s: &StructDef, ir: &SchemaIR, indent: &str) {
        let struct_name = to_csharp_type_name(&s.qname.local);
        if let Some(ref doc) = s.documentation {
            self.emit_docstring(out, doc, indent);
        }

        // XmlRoot attribute if enabled
        if self.options.emit_xml_attributes {
            if let Some(ref ns) = s.qname.namespace {
                writeln!(
                    out,
                    "{}[XmlRoot(\"{}\", Namespace = \"{}\")]",
                    indent, s.qname.local, ns
                )
                .unwrap();
            } else {
                writeln!(out, "{}[XmlRoot(\"{}\")]", indent, s.qname.local).unwrap();
            }
        }

        let record_keyword = match self.options.record_kind {
            CSharpRecordKind::Class => "record",
            CSharpRecordKind::Struct => "readonly record struct",
        };

        // Determine inheritance / interface implementation
        let mut base_clause = Vec::new();
        if let Some(ref base_qname) = s.base_type {
            if matches!(ir.types.get(base_qname), Some(TypeDef::Struct(_))) {
                let base_name = to_csharp_type_name(&base_qname.local);
                base_clause.push(base_name);
            }
        }
        if self.options.emit_validation {
            base_clause.push("IValidatableObject".to_string());
        }

        let implements_str = if base_clause.is_empty() {
            String::new()
        } else {
            format!(" : {}", base_clause.join(", "))
        };

        // Collect fields and parameters
        if s.fields.is_empty() {
            writeln!(
                out,
                "{}public {} {}{};",
                indent, record_keyword, struct_name, implements_str
            )
            .unwrap();
            writeln!(out).unwrap();
            return;
        }

        writeln!(out, "{}public {} {}(", indent, record_keyword, struct_name).unwrap();

        for (i, f) in s.fields.iter().enumerate() {
            let prop_name = to_csharp_property_name(&f.name, Some(&struct_name));
            let field_type = self.map_field_type(f, ir);
            let is_opt = f.cardinality.is_optional()
                || f.nillable
                || (f.cardinality.is_list() && f.cardinality.min_occurs == 0);

            // A parameter in C# primary constructor can only have a default value (= null)
            // if all subsequent parameters also have default values (CS1737).
            let can_have_default = is_opt
                && s.fields[i + 1..].iter().all(|next_f| {
                    next_f.cardinality.is_optional()
                        || next_f.nillable
                        || (next_f.cardinality.is_list() && next_f.cardinality.min_occurs == 0)
                });

            let is_last = i == s.fields.len() - 1;
            let comma = if is_last { "" } else { "," };

            let default_val = if can_have_default { " = null" } else { "" };

            let xml_attr = if self.options.emit_xml_attributes {
                self.build_field_xml_attribute(f, ir)
            } else {
                String::new()
            };

            if let Some(ref doc) = f.documentation {
                self.emit_docstring(out, doc, &format!("{}    ", indent));
            }

            writeln!(
                out,
                "{}    {}{} {}{}{}",
                indent, xml_attr, field_type, prop_name, default_val, comma
            )
            .unwrap();
        }

        writeln!(out, "{}){}", indent, implements_str).unwrap();
        writeln!(out, "{}{{", indent).unwrap();

        // Parameterless constructor for XmlSerializer compatibility
        write!(out, "{}    public {}() : this(", indent, struct_name).unwrap();
        for (i, f) in s.fields.iter().enumerate() {
            let field_type = self.map_field_type(f, ir);
            let is_opt = f.cardinality.is_optional()
                || f.nillable
                || (f.cardinality.is_list() && f.cardinality.min_occurs == 0);
            let default_arg = if is_opt {
                "default".to_string()
            } else {
                format!("default({})!", field_type)
            };

            let comma = if i == s.fields.len() - 1 { "" } else { ", " };
            write!(out, "{}{}", default_arg, comma).unwrap();
        }
        writeln!(out, ") {{ }}\n").unwrap();

        // IValidatableObject implementation
        if self.options.emit_validation {
            self.emit_struct_validator(out, s, indent);
        }

        writeln!(out, "{}}}\n", indent).unwrap();
    }

    fn emit_struct_validator(&self, out: &mut String, s: &StructDef, indent: &str) {
        let struct_name = to_csharp_type_name(&s.qname.local);
        writeln!(
            out,
            "{}    public IEnumerable<ValidationResult> Validate(ValidationContext validationContext)",
            indent
        )
        .unwrap();
        writeln!(out, "{}    {{", indent).unwrap();

        let mut has_checks = false;
        for f in &s.fields {
            let prop_name = to_csharp_property_name(&f.name, Some(&struct_name));
            let is_opt = f.cardinality.is_optional()
                || f.nillable
                || (f.cardinality.is_list() && f.cardinality.min_occurs == 0);

            if let Some(ref facets) = f.facets {
                if is_opt {
                    writeln!(out, "{}        if ({} is not null)", indent, prop_name).unwrap();
                    writeln!(out, "{}        {{", indent).unwrap();
                    self.emit_facet_checks(
                        out,
                        facets,
                        &prop_name,
                        &format!("{}            ", indent),
                    );
                    writeln!(out, "{}        }}", indent).unwrap();
                } else {
                    self.emit_facet_checks(out, facets, &prop_name, &format!("{}        ", indent));
                }
                has_checks = true;
            }
        }

        let _ = has_checks;
        writeln!(out, "{}        yield break;", indent).unwrap();
        writeln!(out, "{}    }}", indent).unwrap();
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
                "{}if ({}.Length < {}) yield return new ValidationResult(\"{} length must be >= {}\", [nameof({})]);",
                indent, target, min_len, target, min_len, target
            )
            .unwrap();
        }
        if let Some(max_len) = facets.max_length {
            writeln!(
                out,
                "{}if ({}.Length > {}) yield return new ValidationResult(\"{} length must be <= {}\", [nameof({})]);",
                indent, target, max_len, target, max_len, target
            )
            .unwrap();
        }
        for pattern in &facets.patterns {
            let escaped = pattern.replace('"', "\\\"");
            writeln!(
                out,
                "{}if (!Regex.IsMatch({}.ToString() ?? \"\", \"^{}$\")) yield return new ValidationResult(\"{} does not match pattern {}\", [nameof({})]);",
                indent, target, escaped, target, escaped, target
            )
            .unwrap();
        }
        if let Some(ref min_inc) = facets.min_inclusive {
            writeln!(
                out,
                "{}if ({} < {}) yield return new ValidationResult(\"{} must be >= {}\", [nameof({})]);",
                indent, target, min_inc, target, min_inc, target
            )
            .unwrap();
        }
        if let Some(ref max_inc) = facets.max_inclusive {
            writeln!(
                out,
                "{}if ({} > {}) yield return new ValidationResult(\"{} must be <= {}\", [nameof({})]);",
                indent, target, max_inc, target, max_inc, target
            )
            .unwrap();
        }
        if let Some(ref min_exc) = facets.min_exclusive {
            writeln!(
                out,
                "{}if ({} <= {}) yield return new ValidationResult(\"{} must be > {}\", [nameof({})]);",
                indent, target, min_exc, target, min_exc, target
            )
            .unwrap();
        }
        if let Some(ref max_exc) = facets.max_exclusive {
            writeln!(
                out,
                "{}if ({} >= {}) yield return new ValidationResult(\"{} must be < {}\", [nameof({})]);",
                indent, target, max_exc, target, max_exc, target
            )
            .unwrap();
        }
    }

    fn map_field_type(&self, f: &FieldDef, ir: &SchemaIR) -> String {
        let base_type = match &f.type_ref {
            TypeRef::Primitive(p) => self.context.map_primitive(*p).to_string(),
            TypeRef::Named(qn) => {
                if let Some(TypeDef::Union(u)) = ir.types.get(qn) {
                    to_csharp_type_name(&u.qname.local)
                } else {
                    to_csharp_type_name(&qn.local)
                }
            }
            TypeRef::Boxed(inner) => self.context.map_type_ref(inner),
            TypeRef::List(inner) => format!("List<{}>", self.context.map_type_ref(inner)),
        };

        if f.cardinality.is_list() {
            if f.cardinality.min_occurs == 0 || f.nillable {
                format!("List<{}>?", base_type)
            } else {
                format!("List<{}>", base_type)
            }
        } else if f.cardinality.is_optional() || f.nillable {
            format!("{}?", base_type)
        } else {
            base_type
        }
    }

    fn build_field_xml_attribute(&self, f: &FieldDef, ir: &SchemaIR) -> String {
        // If the field is a choice (UnionDef), emit [property: XmlElement("branchXml", typeof(BranchType))]
        if let TypeRef::Named(ref qname) = f.type_ref {
            if let Some(TypeDef::Union(u)) = ir.types.get(qname) {
                let choice_name = to_csharp_type_name(&u.qname.local);
                let mut attrs = Vec::new();
                for branch in &u.branches {
                    let variant_name = to_csharp_type_name(&branch.variant_name);
                    attrs.push(format!(
                        "[property: XmlElement(\"{}\", typeof({}.{}))]",
                        branch.xml_name, choice_name, variant_name
                    ));
                }
                if !attrs.is_empty() {
                    return format!("{} ", attrs.join(" "));
                }
            }
        }

        match f.kind {
            FieldKind::Attribute => format!("[property: XmlAttribute(\"{}\")] ", f.xml_name),
            FieldKind::Text => "[property: XmlText] ".to_string(),
            FieldKind::Any => "[property: XmlAnyElement] ".to_string(),
            FieldKind::AnyAttribute => "[property: XmlAnyAttribute] ".to_string(),
            FieldKind::Element => format!("[property: XmlElement(\"{}\")] ", f.xml_name),
        }
    }

    fn emit_root_elements(&self, out: &mut String, ir: &SchemaIR, indent: &str) {
        if !self.options.emit_root_records || ir.elements.is_empty() {
            return;
        }

        for elem in ir.elements.values() {
            let elem_name = to_csharp_type_name(&elem.qname.local);
            let target_type = self.context.map_type_ref(&elem.type_ref);

            if elem_name != target_type {
                if self.options.emit_xml_attributes {
                    if let Some(ref ns) = elem.qname.namespace {
                        writeln!(
                            out,
                            "{}[XmlRoot(\"{}\", Namespace = \"{}\")]",
                            indent, elem.qname.local, ns
                        )
                        .unwrap();
                    } else {
                        writeln!(out, "{}[XmlRoot(\"{}\")]", indent, elem.qname.local).unwrap();
                    }
                }
                writeln!(
                    out,
                    "{}public sealed record {} : {}\n{}{{",
                    indent, elem_name, target_type, indent
                )
                .unwrap();
                writeln!(out, "{}    public {}() : base() {{ }}", indent, elem_name).unwrap();
                writeln!(out, "{}}}\n", indent).unwrap();
            }
        }
    }

    fn emit_docstring(&self, out: &mut String, doc: &str, indent: &str) {
        writeln!(out, "{}/// <summary>", indent).unwrap();
        for line in doc.lines() {
            writeln!(out, "{}/// {}", indent, line.trim()).unwrap();
        }
        writeln!(out, "{}/// </summary>", indent).unwrap();
    }
}
