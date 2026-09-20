use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as FmtWrite;

use heck::{AsLowerCamelCase, AsPascalCase};
use serde::{Deserialize, Serialize};

use crate::codegen::{sanitize_keyword, LanguageContext};
use crate::ir::{
    EnumDef, PrimitiveType, QName, RestrictionFacets, SchemaIR, SimpleTypeDef, StructDef, TypeDef,
    TypeRef, UnionDef,
};

/// Options configuring TypeScript 5+ code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptOptions {
    /// Emit runtime Zod validation schemas alongside static types (default: false)
    pub emit_zod: bool,
    /// Use `interface` instead of `type` alias for complex type structs (default: true)
    pub use_interface: bool,
    /// Mark struct fields with `readonly` modifier (default: false)
    pub readonly_fields: bool,
    /// Emit type aliases for top-level root XML elements (default: true)
    pub emit_root_aliases: bool,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            emit_zod: false,
            use_interface: true,
            readonly_fields: false,
            emit_root_aliases: true,
        }
    }
}

/// Language context adapter for TypeScript 5+.
pub struct TypeScriptLanguageContext;

impl LanguageContext for TypeScriptLanguageContext {
    fn target_language(&self) -> &'static str {
        "typescript"
    }

    fn map_primitive(&self, prim: PrimitiveType) -> &'static str {
        match prim {
            PrimitiveType::Boolean => "boolean",
            PrimitiveType::Float | PrimitiveType::Double | PrimitiveType::Decimal => "number",
            PrimitiveType::Byte
            | PrimitiveType::Short
            | PrimitiveType::Int
            | PrimitiveType::Integer
            | PrimitiveType::Long
            | PrimitiveType::UnsignedByte
            | PrimitiveType::UnsignedShort
            | PrimitiveType::UnsignedInt
            | PrimitiveType::UnsignedLong
            | PrimitiveType::PositiveInteger
            | PrimitiveType::NegativeInteger
            | PrimitiveType::NonPositiveInteger
            | PrimitiveType::NonNegativeInteger => "number",
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
            | PrimitiveType::Entities => "string",
            PrimitiveType::Date
            | PrimitiveType::Time
            | PrimitiveType::DateTime
            | PrimitiveType::Duration
            | PrimitiveType::GYear
            | PrimitiveType::GYearMonth
            | PrimitiveType::GMonth
            | PrimitiveType::GMonthDay
            | PrimitiveType::GDay => "string",
            PrimitiveType::Base64Binary | PrimitiveType::HexBinary => "Uint8Array",
            PrimitiveType::AnyType | PrimitiveType::AnySimpleType => "unknown",
        }
    }

    fn map_type_ref(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => self.map_primitive(*prim).to_string(),
            TypeRef::Named(qname) => to_ts_type_name(&qname.local),
            TypeRef::Boxed(inner) | TypeRef::List(inner) => {
                let inner_str = self.map_type_ref(inner);
                if matches!(type_ref, TypeRef::List(_)) {
                    format!("{}[]", inner_str)
                } else {
                    inner_str
                }
            }
        }
    }
}

/// Convert an XML field name to a valid TypeScript property identifier.
pub fn to_ts_field_identifier(name: &str) -> String {
    let raw = AsLowerCamelCase(name).to_string();
    let sanitized = if raw.is_empty() {
        "field".to_string()
    } else if raw.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{}", raw)
    } else {
        raw
    };
    sanitize_keyword(&sanitized, "ts")
}

/// Convert an XML type name to a PascalCase TypeScript type identifier.
pub fn to_ts_type_name(name: &str) -> String {
    let raw = AsPascalCase(name).to_string();
    let sanitized = if raw.is_empty() {
        "Type".to_string()
    } else if raw.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("Type{}", raw)
    } else {
        raw
    };
    sanitize_keyword(&sanitized, "ts")
}

/// Convert an XML enumeration/choice variant name to a PascalCase TypeScript variant identifier.
pub fn to_ts_variant_name(name: &str) -> String {
    let raw = AsPascalCase(name).to_string();
    if raw.is_empty() {
        "Empty".to_string()
    } else if raw.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("Value{}", raw)
    } else {
        raw
    }
}

/// Code generator producing modern TypeScript 5+ models and optional Zod schemas from SchemaIR.
pub struct TypeScriptCodegen {
    pub options: TypeScriptOptions,
    pub context: TypeScriptLanguageContext,
}

impl TypeScriptCodegen {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            options,
            context: TypeScriptLanguageContext,
        }
    }

    /// Generate a complete, standalone TypeScript module from a SchemaIR graph.
    pub fn generate_module(&self, ir: &SchemaIR) -> String {
        let mut out = String::new();
        let recursive_types = self.find_recursive_types(ir);

        self.emit_header(&mut out, ir);

        let ordered = self.order_types(ir);
        for type_def in ordered {
            match type_def {
                TypeDef::Simple(s) => self.emit_simple_type(&mut out, s),
                TypeDef::Enum(e) => self.emit_enum(&mut out, e),
                TypeDef::Union(u) => self.emit_union(&mut out, u),
                TypeDef::Struct(s) => self.emit_struct(&mut out, s, ir, &recursive_types),
            }
        }

        if self.options.emit_root_aliases {
            self.emit_root_aliases(&mut out, ir);
        }

        out
    }

    fn find_recursive_types(&self, ir: &SchemaIR) -> HashSet<QName> {
        let mut recursive = HashSet::new();
        for (qname, type_def) in &ir.types {
            if let TypeDef::Struct(s) = type_def {
                for f in &s.fields {
                    if f.is_cycle_cut {
                        recursive.insert(qname.clone());
                    }
                }
            }
        }
        recursive
    }

    fn emit_header(&self, out: &mut String, ir: &SchemaIR) {
        out.push_str("/**\n");
        out.push_str(" * Generated by PolyXML Compiler (https://github.com/nth-bailey/PolyXML)\n");
        out.push_str(" * Target: TypeScript 5+\n");
        if let Some(ref ns) = ir.target_namespace {
            let _ = writeln!(out, " * Target Namespace: {}", ns);
        }
        out.push_str(" */\n\n");

        if self.options.emit_zod {
            out.push_str("import { z } from \"zod\";\n\n");
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

        // Topologically sort structs based on inheritance
        let mut ordered_structs: Vec<&'a TypeDef> = Vec::new();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();

        fn visit<'a>(
            s: &'a StructDef,
            ir: &'a SchemaIR,
            visiting: &mut HashSet<QName>,
            visited: &mut HashSet<QName>,
            ordered: &mut Vec<&'a TypeDef>,
        ) {
            if visited.contains(&s.qname) || visiting.contains(&s.qname) {
                return;
            }
            visiting.insert(s.qname.clone());
            if let Some(ref base_qname) = s.base_type {
                if base_qname != &s.qname {
                    if let Some(TypeDef::Struct(base_struct)) = ir.types.get(base_qname) {
                        visit(base_struct, ir, visiting, visited, ordered);
                    }
                }
            }
            for f in &s.fields {
                if !f.is_cycle_cut {
                    match &f.type_ref {
                        TypeRef::Named(target_qname) => {
                            if target_qname != &s.qname {
                                if let Some(TypeDef::Struct(dep_struct)) = ir.types.get(target_qname) {
                                    visit(dep_struct, ir, visiting, visited, ordered);
                                }
                            }
                        }
                        TypeRef::List(inner) | TypeRef::Boxed(inner) => {
                            if let TypeRef::Named(target_qname) = inner.as_ref() {
                                if target_qname != &s.qname {
                                    if let Some(TypeDef::Struct(dep_struct)) = ir.types.get(target_qname) {
                                        visit(dep_struct, ir, visiting, visited, ordered);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            visiting.remove(&s.qname);
            visited.insert(s.qname.clone());
            ordered.push(ir.types.get(&s.qname).unwrap());
        }

        for s in structs {
            visit(s, ir, &mut visiting, &mut visited, &mut ordered_structs);
        }

        let mut res = Vec::new();
        res.extend(simples);
        res.extend(enums);
        res.extend(unions);
        res.extend(ordered_structs);
        res
    }

    fn emit_simple_type(&self, out: &mut String, s: &SimpleTypeDef) {
        let ts_name = to_ts_type_name(&s.qname.local);
        let base_type = self.context.map_type_ref(&s.base_type);

        if let Some(ref doc) = s.documentation {
            self.emit_docstring(out, doc, "");
        }

        let _ = writeln!(out, "export type {} = {};\n", ts_name, base_type);

        if self.options.emit_zod {
            let schema_name = format!("{}Schema", ts_name);
            let mut zod_expr = self.zod_expr_for_type(&s.base_type);
            self.apply_facets_to_zod(&mut zod_expr, &s.facets, &s.base_type);
            let _ = writeln!(out, "export const {} = {};\n", schema_name, zod_expr);
        }
    }

    fn emit_enum(&self, out: &mut String, e: &EnumDef) {
        let ts_name = to_ts_type_name(&e.qname.local);

        if let Some(ref doc) = e.documentation {
            self.emit_docstring(out, doc, "");
        }

        // Companion const object for runtime lookups & autocomplete
        let _ = writeln!(out, "export const {} = {{", ts_name);
        for v in &e.variants {
            let variant_key = to_ts_variant_name(&v.name);
            let _ = writeln!(out, "  {}: {:?},", variant_key, v.value);
        }
        out.push_str("} as const;\n\n");

        // Type definition from const object values
        let _ = writeln!(
            out,
            "export type {} = (typeof {})[keyof typeof {}];\n",
            ts_name, ts_name, ts_name
        );

        if self.options.emit_zod {
            let schema_name = format!("{}Schema", ts_name);
            let literals = e
                .variants
                .iter()
                .map(|v| format!("{:?}", v.value))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(
                out,
                "export const {} = z.enum([{}]);\n",
                schema_name, literals
            );
        }
    }

    fn emit_union(&self, out: &mut String, u: &UnionDef) {
        let ts_name = to_ts_type_name(&u.qname.local);

        if let Some(ref doc) = u.documentation {
            self.emit_docstring(out, doc, "");
        }

        let _ = writeln!(out, "export type {} =", ts_name);
        for branch in &u.branches {
            let kind_name = to_ts_variant_name(&branch.variant_name);
            let branch_type = self.context.map_type_ref(&branch.type_ref);
            let _ = writeln!(
                out,
                "  | {{ readonly kind: {:?}; readonly value: {}; }}",
                kind_name, branch_type
            );
        }
        out.push_str(";\n\n");

        if self.options.emit_zod {
            let schema_name = format!("{}Schema", ts_name);
            let _ = writeln!(
                out,
                "export const {} = z.discriminatedUnion(\"kind\", [",
                schema_name
            );
            for branch in &u.branches {
                let kind_name = to_ts_variant_name(&branch.variant_name);
                let branch_zod = self.zod_expr_for_type(&branch.type_ref);
                let _ = writeln!(
                    out,
                    "  z.object({{ kind: z.literal({:?}), value: {} }}),",
                    kind_name, branch_zod
                );
            }
            out.push_str("]);\n\n");
        }
    }

    fn emit_struct(
        &self,
        out: &mut String,
        s: &StructDef,
        ir: &SchemaIR,
        recursive_types: &HashSet<QName>,
    ) {
        let ts_name = to_ts_type_name(&s.qname.local);

        if let Some(ref doc) = s.documentation {
            self.emit_docstring(out, doc, "");
        }

        let mut extends_clause = String::new();
        if let Some(ref base) = s.base_type {
            let base_name = to_ts_type_name(&base.local);
            extends_clause = format!(" extends {}", base_name);
        }

        let readonly_prefix = if self.options.readonly_fields {
            "readonly "
        } else {
            ""
        };

        if self.options.use_interface {
            let _ = writeln!(out, "export interface {}{} {{", ts_name, extends_clause);
        } else if extends_clause.is_empty() {
            let _ = writeln!(out, "export type {} = {{", ts_name);
        } else {
            let base_name = to_ts_type_name(&s.base_type.as_ref().unwrap().local);
            let _ = writeln!(out, "export type {} = {} & {{", ts_name, base_name);
        }

        let mut seen_fields = HashSet::new();

        for field in &s.fields {
            let field_id = self.unique_field_name(&field.name, &mut seen_fields);
            let is_optional = field.cardinality.is_optional();
            let is_list = field.cardinality.is_list() || field.type_ref.is_list();

            if let Some(ref doc) = field.documentation {
                self.emit_docstring(out, doc, "  ");
            }

            let type_str = self.format_field_type(&field.type_ref, is_list, field.nillable);
            let optional_marker = if is_optional { "?" } else { "" };

            let _ = writeln!(
                out,
                "  {}{}{}: {};",
                readonly_prefix, field_id, optional_marker, type_str
            );
        }

        out.push_str("}\n\n");

        if self.options.emit_zod {
            self.emit_zod_struct(out, s, ts_name.as_str(), ir, recursive_types);
        }
    }

    fn emit_zod_struct(
        &self,
        out: &mut String,
        s: &StructDef,
        ts_name: &str,
        _ir: &SchemaIR,
        recursive_types: &HashSet<QName>,
    ) {
        let schema_name = format!("{}Schema", ts_name);
        let is_recursive = recursive_types.contains(&s.qname);

        if is_recursive {
            let _ = writeln!(
                out,
                "export const {}: z.ZodType<{}> = z.lazy(() => z.object({{",
                schema_name, ts_name
            );
        } else {
            let _ = writeln!(out, "export const {} = z.object({{", schema_name);
        }

        let mut seen_fields = HashSet::new();
        for field in &s.fields {
            let field_id = self.unique_field_name(&field.name, &mut seen_fields);
            let is_optional = field.cardinality.is_optional();
            let is_list = field.cardinality.is_list() || field.type_ref.is_list();

            let mut zod_expr = self.zod_expr_for_type(&field.type_ref);

            if let Some(ref facets) = field.facets {
                self.apply_facets_to_zod(&mut zod_expr, facets, &field.type_ref);
            }

            if is_list {
                zod_expr = format!("z.array({})", zod_expr);
            }

            if field.nillable {
                zod_expr = format!("{}.nullable()", zod_expr);
            }

            if is_optional {
                zod_expr = format!("{}.optional()", zod_expr);
            }

            let _ = writeln!(out, "  {}: {},", field_id, zod_expr);
        }

        if is_recursive {
            out.push_str("}));\n\n");
        } else {
            out.push_str("});\n\n");
        }
    }

    fn format_field_type(&self, type_ref: &TypeRef, is_list: bool, nillable: bool) -> String {
        let base_type = match type_ref {
            TypeRef::List(inner) => self.context.map_type_ref(inner),
            other => self.context.map_type_ref(other),
        };

        let mut res = if is_list {
            format!("{}[]", base_type)
        } else {
            base_type
        };

        if nillable {
            res = format!("{} | null", res);
        }

        res
    }

    fn zod_expr_for_type(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Primitive(prim) => match prim {
                PrimitiveType::Boolean => "z.boolean()".into(),
                PrimitiveType::Float | PrimitiveType::Double | PrimitiveType::Decimal => {
                    "z.number()".into()
                }
                PrimitiveType::Byte
                | PrimitiveType::Short
                | PrimitiveType::Int
                | PrimitiveType::Integer
                | PrimitiveType::Long
                | PrimitiveType::UnsignedByte
                | PrimitiveType::UnsignedShort
                | PrimitiveType::UnsignedInt
                | PrimitiveType::UnsignedLong
                | PrimitiveType::PositiveInteger
                | PrimitiveType::NegativeInteger
                | PrimitiveType::NonPositiveInteger
                | PrimitiveType::NonNegativeInteger => "z.number().int()".into(),
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
                | PrimitiveType::Entities => "z.string()".into(),
                PrimitiveType::Date
                | PrimitiveType::Time
                | PrimitiveType::DateTime
                | PrimitiveType::Duration
                | PrimitiveType::GYear
                | PrimitiveType::GYearMonth
                | PrimitiveType::GMonth
                | PrimitiveType::GMonthDay
                | PrimitiveType::GDay => "z.string()".into(),
                PrimitiveType::Base64Binary | PrimitiveType::HexBinary => {
                    "z.instanceof(Uint8Array)".into()
                }
                PrimitiveType::AnyType | PrimitiveType::AnySimpleType => "z.unknown()".into(),
            },
            TypeRef::Named(qname) => format!("{}Schema", to_ts_type_name(&qname.local)),
            TypeRef::Boxed(inner) => self.zod_expr_for_type(inner),
            TypeRef::List(inner) => format!("z.array({})", self.zod_expr_for_type(inner)),
        }
    }

    fn apply_facets_to_zod(
        &self,
        zod_expr: &mut String,
        facets: &RestrictionFacets,
        type_ref: &TypeRef,
    ) {
        let is_string = match type_ref {
            TypeRef::Primitive(prim) => matches!(
                prim,
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
            ),
            _ => false,
        };

        let is_num = match type_ref {
            TypeRef::Primitive(prim) => matches!(
                prim,
                PrimitiveType::Byte
                    | PrimitiveType::Short
                    | PrimitiveType::Int
                    | PrimitiveType::Integer
                    | PrimitiveType::Long
                    | PrimitiveType::UnsignedByte
                    | PrimitiveType::UnsignedShort
                    | PrimitiveType::UnsignedInt
                    | PrimitiveType::UnsignedLong
                    | PrimitiveType::PositiveInteger
                    | PrimitiveType::NegativeInteger
                    | PrimitiveType::NonPositiveInteger
                    | PrimitiveType::NonNegativeInteger
                    | PrimitiveType::Float
                    | PrimitiveType::Double
                    | PrimitiveType::Decimal
            ),
            _ => false,
        };

        if is_string {
            if let Some(min_len) = facets.min_length {
                zod_expr.push_str(&format!(".min({})", min_len));
            }
            if let Some(max_len) = facets.max_length {
                zod_expr.push_str(&format!(".max({})", max_len));
            }
            if let Some(length) = facets.length {
                zod_expr.push_str(&format!(".length({})", length));
            }
            for pat in &facets.patterns {
                zod_expr.push_str(&format!(".regex(new RegExp({:?}))", pat));
            }
        }

        if is_num {
            if let Some(ref min_inc) = facets.min_inclusive {
                zod_expr.push_str(&format!(".gte({})", min_inc));
            }
            if let Some(ref max_inc) = facets.max_inclusive {
                zod_expr.push_str(&format!(".lte({})", max_inc));
            }
            if let Some(ref min_exc) = facets.min_exclusive {
                zod_expr.push_str(&format!(".gt({})", min_exc));
            }
            if let Some(ref max_exc) = facets.max_exclusive {
                zod_expr.push_str(&format!(".lt({})", max_exc));
            }
        }
    }

    fn emit_docstring(&self, out: &mut String, doc: &str, indent: &str) {
        let clean = doc.trim();
        if clean.contains('\n') {
            let _ = writeln!(out, "{}/**", indent);
            for line in clean.lines() {
                let _ = writeln!(out, "{} * {}", indent, line.trim());
            }
            let _ = writeln!(out, "{} */", indent);
        } else {
            let _ = writeln!(out, "{}/** {} */", indent, clean);
        }
    }

    fn emit_root_aliases(&self, out: &mut String, ir: &SchemaIR) {
        let mut declared_names = BTreeSet::new();
        for td in ir.types.values() {
            declared_names.insert(to_ts_type_name(&td.qname().local));
        }

        for element in ir.elements.values() {
            let el_name = to_ts_type_name(&element.qname.local);
            if !declared_names.contains(&el_name) {
                let target_type = self.context.map_type_ref(&element.type_ref);
                if el_name != target_type {
                    let _ = writeln!(out, "export type {} = {};", el_name, target_type);
                    if self.options.emit_zod {
                        let target_schema = format!("{}Schema", target_type);
                        let _ =
                            writeln!(out, "export const {}Schema = {};", el_name, target_schema);
                    }
                    declared_names.insert(el_name);
                }
            }
        }
    }

    fn unique_field_name(&self, name: &str, seen: &mut HashSet<String>) -> String {
        let base = to_ts_field_identifier(name);
        let mut candidate = base.clone();
        let mut counter = 1;
        while seen.contains(&candidate) {
            counter += 1;
            candidate = format!("{}_{}", base, counter);
        }
        seen.insert(candidate.clone());
        candidate
    }
}
