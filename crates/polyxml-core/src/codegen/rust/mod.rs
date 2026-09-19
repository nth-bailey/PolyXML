use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as FmtWrite;

use heck::{AsPascalCase, AsSnakeCase};
use serde::{Deserialize, Serialize};

use crate::codegen::{sanitize_keyword, LanguageContext};
use crate::ir::{
    EnumDef, FieldDef, FieldKind, PrimitiveType, QName, SchemaIR, SimpleTypeDef, StructDef,
    TypeDef, TypeRef, UnionDef,
};

/// Options configuring Rust 2021/2024 code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustOptions {
    /// Use zero-copy Cow<'a, str> and Cow<'a, [u8]> (default: true).
    pub zero_copy: bool,
    /// Derive serde::{Serialize, Deserialize} (default: true).
    pub derive_serde: bool,
    /// Derive Default when applicable (default: true).
    pub derive_default: bool,
    /// Emit #[polyxml(...)] field attributes (default: true).
    pub emit_polyxml_attrs: bool,
    /// Emit root element type aliases (default: true).
    pub emit_root_aliases: bool,
}

impl Default for RustOptions {
    fn default() -> Self {
        Self {
            zero_copy: true,
            derive_serde: true,
            derive_default: true,
            emit_polyxml_attrs: true,
            emit_root_aliases: true,
        }
    }
}

/// Language context adapter for Rust 2021/2024.
pub struct RustLanguageContext {
    zero_copy: bool,
}

impl RustLanguageContext {
    pub fn new(zero_copy: bool) -> Self {
        Self { zero_copy }
    }
}

impl LanguageContext for RustLanguageContext {
    fn target_language(&self) -> &'static str {
        "rust"
    }

    fn map_primitive(&self, prim: PrimitiveType) -> &'static str {
        match prim {
            PrimitiveType::String
            | PrimitiveType::NormalizedString
            | PrimitiveType::Token
            | PrimitiveType::Language
            | PrimitiveType::NMTOKEN
            | PrimitiveType::NMTOKENS
            | PrimitiveType::Name
            | PrimitiveType::NCName
            | PrimitiveType::Id
            | PrimitiveType::IdRef
            | PrimitiveType::IdRefs
            | PrimitiveType::Entity
            | PrimitiveType::Entities
            | PrimitiveType::DateTime
            | PrimitiveType::Date
            | PrimitiveType::Time
            | PrimitiveType::Duration
            | PrimitiveType::GYearMonth
            | PrimitiveType::GYear
            | PrimitiveType::GMonthDay
            | PrimitiveType::GDay
            | PrimitiveType::GMonth
            | PrimitiveType::AnyUri
            | PrimitiveType::QName
            | PrimitiveType::AnyType
            | PrimitiveType::AnySimpleType => {
                if self.zero_copy {
                    "Cow<'a, str>"
                } else {
                    "String"
                }
            }

            PrimitiveType::Boolean => "bool",

            PrimitiveType::Decimal | PrimitiveType::Double => "f64",
            PrimitiveType::Float => "f32",

            PrimitiveType::Integer
            | PrimitiveType::Long
            | PrimitiveType::NonPositiveInteger
            | PrimitiveType::NegativeInteger => "i64",
            PrimitiveType::Int => "i32",
            PrimitiveType::Short => "i16",
            PrimitiveType::Byte => "i8",

            PrimitiveType::PositiveInteger
            | PrimitiveType::NonNegativeInteger
            | PrimitiveType::UnsignedLong => "u64",
            PrimitiveType::UnsignedInt => "u32",
            PrimitiveType::UnsignedShort => "u16",
            PrimitiveType::UnsignedByte => "u8",

            PrimitiveType::HexBinary | PrimitiveType::Base64Binary => {
                if self.zero_copy {
                    "Cow<'a, [u8]>"
                } else {
                    "Vec<u8>"
                }
            }
        }
    }

    fn map_type_ref(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => self.map_primitive(*prim).to_string(),
            TypeRef::Named(qname) => AsPascalCase(&qname.local).to_string(),
            TypeRef::Boxed(inner) => format!("Box<{}>", self.map_type_ref(inner)),
            TypeRef::List(inner) => format!("Vec<{}>", self.map_type_ref(inner)),
        }
    }
}

/// Converts an arbitrary string into a safe, valid Rust enum variant identifier (PascalCase).
pub fn to_rust_variant_identifier(val: &str) -> String {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return "Empty".to_string();
    }

    let cleaned: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();

    let pascal = AsPascalCase(&cleaned).to_string();
    let identifier = if pascal.is_empty() {
        "Empty".to_string()
    } else {
        pascal
    };

    if identifier
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("Value{}", identifier)
    } else {
        identifier
    }
}

/// Sanitizes a Rust field name to snake_case and raw identifier if reserved.
pub fn to_rust_field_identifier(name: &str) -> String {
    let snake = AsSnakeCase(name).to_string();
    let safe_name = if snake
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("_{}", snake)
    } else if snake.is_empty() {
        "value".to_string()
    } else {
        snake
    };

    sanitize_keyword(&safe_name, "rust")
}

/// Pure-Rust code generator emitting zero-copy / low-allocation Rust 2021/2024 data structures.
pub struct RustCodegen {
    options: RustOptions,
    context: RustLanguageContext,
}

impl RustCodegen {
    pub fn new(options: RustOptions) -> Self {
        let zero_copy = options.zero_copy;
        Self {
            options,
            context: RustLanguageContext::new(zero_copy),
        }
    }

    pub fn generate_module(&self, ir: &SchemaIR) -> String {
        let mut out = String::new();

        out.push_str("// Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\n");
        out.push_str(
            "#![allow(dead_code, unused_imports, non_camel_case_types, non_snake_case)]\n\n",
        );

        let types_with_lifetime = if self.options.zero_copy {
            self.compute_types_with_lifetime(ir)
        } else {
            HashSet::new()
        };

        self.emit_imports(&mut out, !types_with_lifetime.is_empty());

        // Sort types topologically (base types before derived types)
        let sorted_types = self.order_types(ir);

        for type_def in sorted_types {
            out.push('\n');
            match type_def {
                TypeDef::Simple(s) => self.emit_simple_type(&mut out, s, &types_with_lifetime),
                TypeDef::Enum(e) => self.emit_enum(&mut out, e),
                TypeDef::Union(u) => self.emit_union(&mut out, u, &types_with_lifetime),
                TypeDef::Struct(s) => self.emit_struct(&mut out, s, &types_with_lifetime),
            }
        }

        if self.options.emit_root_aliases {
            self.emit_root_aliases(&mut out, ir, &types_with_lifetime);
        }

        out
    }

    /// Fixed-point analysis determining which types in SchemaIR require a lifetime parameter `<'a>`.
    fn compute_types_with_lifetime(&self, ir: &SchemaIR) -> HashSet<QName> {
        let mut requires_lifetime = HashSet::new();

        fn primitive_has_lifetime(prim: PrimitiveType) -> bool {
            matches!(
                prim,
                PrimitiveType::String
                    | PrimitiveType::NormalizedString
                    | PrimitiveType::Token
                    | PrimitiveType::Language
                    | PrimitiveType::NMTOKEN
                    | PrimitiveType::NMTOKENS
                    | PrimitiveType::Name
                    | PrimitiveType::NCName
                    | PrimitiveType::Id
                    | PrimitiveType::IdRef
                    | PrimitiveType::IdRefs
                    | PrimitiveType::Entity
                    | PrimitiveType::Entities
                    | PrimitiveType::DateTime
                    | PrimitiveType::Date
                    | PrimitiveType::Time
                    | PrimitiveType::Duration
                    | PrimitiveType::GYearMonth
                    | PrimitiveType::GYear
                    | PrimitiveType::GMonthDay
                    | PrimitiveType::GDay
                    | PrimitiveType::GMonth
                    | PrimitiveType::AnyUri
                    | PrimitiveType::QName
                    | PrimitiveType::HexBinary
                    | PrimitiveType::Base64Binary
                    | PrimitiveType::AnyType
                    | PrimitiveType::AnySimpleType
            )
        }

        fn typeref_has_lifetime(type_ref: &TypeRef, current_set: &HashSet<QName>) -> bool {
            match type_ref {
                TypeRef::Primitive(prim) => primitive_has_lifetime(*prim),
                TypeRef::Named(qname) => current_set.contains(qname),
                TypeRef::Boxed(inner) | TypeRef::List(inner) => {
                    typeref_has_lifetime(inner, current_set)
                }
            }
        }

        // Fixed-point iteration
        loop {
            let mut changed = false;

            for (qname, type_def) in &ir.types {
                if requires_lifetime.contains(qname) {
                    continue;
                }

                let needs = match type_def {
                    TypeDef::Simple(s) => typeref_has_lifetime(&s.base_type, &requires_lifetime),
                    TypeDef::Enum(_) => false, // unit variants do not borrow
                    TypeDef::Union(u) => u
                        .branches
                        .iter()
                        .any(|b| typeref_has_lifetime(&b.type_ref, &requires_lifetime)),
                    TypeDef::Struct(s) => s
                        .fields
                        .iter()
                        .any(|f| typeref_has_lifetime(&f.type_ref, &requires_lifetime)),
                };

                if needs {
                    requires_lifetime.insert(qname.clone());
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        requires_lifetime
    }

    fn emit_imports(&self, out: &mut String, has_borrowed_types: bool) {
        if self.options.zero_copy && has_borrowed_types {
            out.push_str("use std::borrow::Cow;\n");
        }
        if self.options.derive_serde {
            out.push_str("use serde::{Deserialize, Serialize};\n");
        }
    }

    fn order_types<'a>(&self, ir: &'a SchemaIR) -> Vec<&'a TypeDef> {
        let mut simples = Vec::new();
        let mut enums = Vec::new();
        let mut unions = Vec::new();
        let mut structs: Vec<&'a StructDef> = Vec::new();

        for type_def in ir.types.values() {
            match type_def {
                TypeDef::Simple(_) => simples.push(type_def),
                TypeDef::Enum(_) => enums.push(type_def),
                TypeDef::Union(_) => unions.push(type_def),
                TypeDef::Struct(s) => structs.push(s),
            }
        }

        let mut ordered_structs: Vec<&'a TypeDef> = Vec::new();
        let mut visited = HashSet::new();

        fn visit<'a>(
            s: &'a StructDef,
            ir: &'a SchemaIR,
            visited: &mut HashSet<QName>,
            ordered: &mut Vec<&'a TypeDef>,
        ) {
            if visited.contains(&s.qname) {
                return;
            }
            if let Some(ref base_qname) = s.base_type {
                if let Some(TypeDef::Struct(parent)) = ir.find_type(base_qname) {
                    visit(parent, ir, visited, ordered);
                }
            }
            visited.insert(s.qname.clone());
            if let Some(td) = ir.find_type(&s.qname) {
                ordered.push(td);
            }
        }

        for s in &structs {
            visit(s, ir, &mut visited, &mut ordered_structs);
        }

        let mut result = Vec::new();
        result.extend(simples);
        result.extend(enums);
        result.extend(unions);
        result.extend(ordered_structs);
        result
    }

    fn emit_simple_type(
        &self,
        out: &mut String,
        s: &SimpleTypeDef,
        types_with_lifetime: &HashSet<QName>,
    ) {
        let type_name = AsPascalCase(&s.qname.local).to_string();
        let needs_lifetime = types_with_lifetime.contains(&s.qname);

        if let Some(ref doc) = s.documentation {
            let _ = writeln!(out, "/// {}", doc.trim());
        }

        let base_type = self.format_rust_type_ref(&s.base_type, types_with_lifetime);
        if needs_lifetime {
            let _ = writeln!(out, "pub type {}<'a> = {};", type_name, base_type);
        } else {
            let _ = writeln!(out, "pub type {} = {};", type_name, base_type);
        }
    }

    fn emit_enum(&self, out: &mut String, e: &EnumDef) {
        let enum_name = AsPascalCase(&e.qname.local).to_string();

        if let Some(ref doc) = e.documentation {
            let _ = writeln!(out, "/// {}", doc.trim());
        }

        let mut derives = vec!["Debug", "Clone", "Copy", "PartialEq", "Eq", "Hash"];
        if self.options.derive_serde {
            derives.push("Serialize");
            derives.push("Deserialize");
        }

        let _ = writeln!(out, "#[derive({})]", derives.join(", "));
        let _ = writeln!(out, "pub enum {} {{", enum_name);

        let mut seen_variants = HashSet::new();
        let mut variant_map = Vec::new();

        for variant in &e.variants {
            let mut var_id = to_rust_variant_identifier(&variant.name);
            let mut counter = 1;
            while seen_variants.contains(&var_id) {
                counter += 1;
                var_id = format!("{}{}", to_rust_variant_identifier(&variant.name), counter);
            }
            seen_variants.insert(var_id.clone());

            if let Some(ref doc) = variant.documentation {
                let _ = writeln!(out, "    /// {}", doc.trim());
            }
            if self.options.derive_serde {
                let _ = writeln!(out, "    #[serde(rename = \"{}\")]", variant.value);
            }
            let _ = writeln!(out, "    {},", var_id);
            variant_map.push((var_id, variant.value.clone()));
        }

        out.push_str("}\n\n");

        // Implement helper methods: as_str(), FromStr, Display
        let _ = writeln!(out, "impl {} {{", enum_name);
        out.push_str("    pub fn as_str(&self) -> &'static str {\n");
        out.push_str("        match self {\n");
        for (var_id, val) in &variant_map {
            let _ = writeln!(out, "            Self::{} => \"{}\",", var_id, val);
        }
        if variant_map.is_empty() {
            out.push_str("            _ => \"\",\n");
        }
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // std::str::FromStr
        let _ = writeln!(out, "impl std::str::FromStr for {} {{", enum_name);
        out.push_str("    type Err = String;\n\n");
        out.push_str("    fn from_str(s: &str) -> Result<Self, Self::Err> {\n");
        out.push_str("        match s {\n");
        for (var_id, val) in &variant_map {
            let _ = writeln!(out, "            \"{}\" => Ok(Self::{}),", val, var_id);
        }
        let _ = writeln!(
            out,
            "            _ => Err(format!(\"Unknown {} variant: {{}}\", s)),",
            enum_name
        );
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // std::fmt::Display
        let _ = writeln!(out, "impl std::fmt::Display for {} {{", enum_name);
        out.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        out.push_str("        write!(f, \"{}\", self.as_str())\n");
        out.push_str("    }\n");
        out.push_str("}\n");
    }

    fn emit_union(&self, out: &mut String, u: &UnionDef, types_with_lifetime: &HashSet<QName>) {
        let union_name = AsPascalCase(&u.qname.local).to_string();
        let needs_lifetime = types_with_lifetime.contains(&u.qname);

        if let Some(ref doc) = u.documentation {
            let _ = writeln!(out, "/// {}", doc.trim());
        }

        let mut derives = vec!["Debug", "Clone", "PartialEq"];
        if self.options.derive_serde {
            derives.push("Serialize");
            derives.push("Deserialize");
        }

        let _ = writeln!(out, "#[derive({})]", derives.join(", "));
        let type_signature = if needs_lifetime {
            format!("{}<'a>", union_name)
        } else {
            union_name
        };

        let _ = writeln!(out, "pub enum {} {{", type_signature);

        let mut seen_variants = HashSet::new();
        for branch in &u.branches {
            let mut var_id = to_rust_variant_identifier(&branch.variant_name);
            let mut counter = 1;
            while seen_variants.contains(&var_id) {
                counter += 1;
                var_id = format!(
                    "{}{}",
                    to_rust_variant_identifier(&branch.variant_name),
                    counter
                );
            }
            seen_variants.insert(var_id.clone());

            let branch_type = self.format_rust_type_ref(&branch.type_ref, types_with_lifetime);

            if let Some(ref doc) = branch.documentation {
                let _ = writeln!(out, "    /// {}", doc.trim());
            }
            if self.options.derive_serde {
                let _ = writeln!(out, "    #[serde(rename = \"{}\")]", branch.xml_name);
            }
            if self.options.emit_polyxml_attrs {
                let _ = writeln!(out, "    #[polyxml(element = \"{}\")]", branch.xml_name);
            }
            let _ = writeln!(out, "    {}({}),", var_id, branch_type);
        }

        out.push_str("}\n");
    }

    fn emit_struct(&self, out: &mut String, s: &StructDef, types_with_lifetime: &HashSet<QName>) {
        let struct_name = AsPascalCase(&s.qname.local).to_string();
        let needs_lifetime = types_with_lifetime.contains(&s.qname);

        if let Some(ref doc) = s.documentation {
            let _ = writeln!(out, "/// {}", doc.trim());
        }

        let mut derives = vec!["Debug", "Clone", "PartialEq"];
        if self.options.derive_default {
            derives.push("Default");
        }
        if self.options.derive_serde {
            derives.push("Serialize");
            derives.push("Deserialize");
        }

        let _ = writeln!(out, "#[derive({})]", derives.join(", "));

        let struct_decl = if needs_lifetime {
            format!("pub struct {}<'a>", struct_name)
        } else {
            format!("pub struct {}", struct_name)
        };

        let _ = writeln!(out, "{} {{", struct_decl);

        let mut seen_fields = HashSet::new();
        for field in &s.fields {
            let rust_name = self.unique_rust_field_name(&field.name, &mut seen_fields);
            self.emit_struct_field(out, field, &rust_name, types_with_lifetime);
        }

        out.push_str("}\n");
    }

    fn unique_rust_field_name(&self, name: &str, seen: &mut HashSet<String>) -> String {
        let mut candidate = to_rust_field_identifier(name);
        let mut counter = 1;
        while seen.contains(&candidate) {
            counter += 1;
            candidate = format!("{}_{}", to_rust_field_identifier(name), counter);
        }
        seen.insert(candidate.clone());
        candidate
    }

    fn emit_struct_field(
        &self,
        out: &mut String,
        field: &FieldDef,
        rust_name: &str,
        types_with_lifetime: &HashSet<QName>,
    ) {
        if let Some(ref doc) = field.documentation {
            let _ = writeln!(out, "    /// {}", doc.trim());
        }

        let is_list = field.cardinality.is_list() || field.type_ref.is_list();
        let is_optional = field.cardinality.is_optional() || field.nillable;

        // PolyXML attribute
        if self.options.emit_polyxml_attrs {
            let kind_attr = match field.kind {
                FieldKind::Element => format!("element = \"{}\"", field.xml_name),
                FieldKind::Attribute => format!("attribute = \"{}\"", field.xml_name),
                FieldKind::Text => "text".to_string(),
                FieldKind::Any | FieldKind::AnyAttribute => "wildcard".to_string(),
            };
            let _ = writeln!(out, "    #[polyxml({})]", kind_attr);
        }

        // Serde attribute
        if self.options.derive_serde {
            let mut serde_parts = Vec::new();
            if field.xml_name != rust_name.strip_prefix("r#").unwrap_or(rust_name) {
                serde_parts.push(format!("rename = \"{}\"", field.xml_name));
            }
            if is_optional {
                serde_parts.push("default".to_string());
                serde_parts.push("skip_serializing_if = \"Option::is_none\"".to_string());
            } else if is_list {
                serde_parts.push("default".to_string());
                serde_parts.push("skip_serializing_if = \"Vec::is_empty\"".to_string());
            }

            if !serde_parts.is_empty() {
                let _ = writeln!(out, "    #[serde({})]", serde_parts.join(", "));
            }
        }

        // Compute Rust type
        let inner_ref = match &field.type_ref {
            TypeRef::List(inner) => inner.as_ref(),
            other => other,
        };

        let is_boxed = field.is_cycle_cut || field.type_ref.is_boxed();
        let formatted_inner = self.format_rust_type_ref(inner_ref, types_with_lifetime);

        let final_type = if is_list {
            format!("Vec<{}>", formatted_inner)
        } else if is_optional {
            if is_boxed {
                format!("Option<Box<{}>>", formatted_inner)
            } else {
                format!("Option<{}>", formatted_inner)
            }
        } else if is_boxed {
            format!("Box<{}>", formatted_inner)
        } else {
            formatted_inner
        };

        let _ = writeln!(out, "    pub {}: {},", rust_name, final_type);
    }

    fn format_rust_type_ref(
        &self,
        type_ref: &TypeRef,
        types_with_lifetime: &HashSet<QName>,
    ) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => self.context.map_primitive(*prim).to_string(),
            TypeRef::Named(qname) => {
                let name = AsPascalCase(&qname.local).to_string();
                if self.options.zero_copy && types_with_lifetime.contains(qname) {
                    format!("{}<'a>", name)
                } else {
                    name
                }
            }
            TypeRef::Boxed(inner) => {
                format!(
                    "Box<{}>",
                    self.format_rust_type_ref(inner, types_with_lifetime)
                )
            }
            TypeRef::List(inner) => {
                format!(
                    "Vec<{}>",
                    self.format_rust_type_ref(inner, types_with_lifetime)
                )
            }
        }
    }

    fn emit_root_aliases(
        &self,
        out: &mut String,
        ir: &SchemaIR,
        types_with_lifetime: &HashSet<QName>,
    ) {
        let mut declared_names = BTreeSet::new();
        for td in ir.types.values() {
            declared_names.insert(AsPascalCase(&td.qname().local).to_string());
        }

        for element in ir.elements.values() {
            let el_name = AsPascalCase(&element.qname.local).to_string();
            if !declared_names.contains(&el_name) {
                let target_type = self.format_rust_type_ref(&element.type_ref, types_with_lifetime);
                let target_clean = target_type
                    .split('<')
                    .next()
                    .unwrap_or(&target_type)
                    .to_string();

                if el_name != target_clean {
                    if target_type.contains("'a") {
                        let _ = writeln!(out, "\npub type {}<'a> = {};", el_name, target_type);
                    } else {
                        let _ = writeln!(out, "\npub type {} = {};", el_name, target_type);
                    }
                    declared_names.insert(el_name);
                }
            }
        }
    }
}
