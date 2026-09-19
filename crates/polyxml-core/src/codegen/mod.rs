use std::collections::HashSet;
use std::sync::OnceLock;

use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsShoutySnakeCase, AsSnakeCase};
use minijinja::{Environment, Error as JinjaError, Value};
use thiserror::Error;

use crate::ir::{PrimitiveType, TypeRef};

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Template engine error: {0}")]
    Template(#[from] JinjaError),

    #[error("Language generation error: {0}")]
    Generation(String),
}

/// Adapter trait feeding normalized schema context and language conventions into templates.
pub trait LanguageContext: Send + Sync {
    /// Return the target language name (e.g. "python", "rust", "cpp", "java", "ts", "go", "csharp").
    fn target_language(&self) -> &'static str;

    /// Sanitize an identifier to avoid collisions with reserved keywords.
    fn sanitize_identifier(&self, id: &str) -> String {
        sanitize_keyword(id, self.target_language())
    }

    /// Map a primitive XSD type to the target language's native type representation.
    fn map_primitive(&self, prim: PrimitiveType) -> &'static str;

    /// Map a canonical TypeRef to the target language's type string.
    fn map_type_ref(&self, type_ref: &TypeRef) -> String;
}

/// Create a pre-configured MiniJinja environment with PolyXML case filters and keyword sanitization.
pub fn create_template_engine() -> Environment<'static> {
    let mut env = Environment::new();

    // Register case transformation filters
    env.add_filter("pascal_case", |val: Value| -> String {
        AsPascalCase(val.as_str().unwrap_or_default()).to_string()
    });

    env.add_filter("snake_case", |val: Value| -> String {
        AsSnakeCase(val.as_str().unwrap_or_default()).to_string()
    });

    env.add_filter("camel_case", |val: Value| -> String {
        AsLowerCamelCase(val.as_str().unwrap_or_default()).to_string()
    });

    env.add_filter("screaming_snake_case", |val: Value| -> String {
        AsShoutySnakeCase(val.as_str().unwrap_or_default()).to_string()
    });

    env.add_filter("kebab_case", |val: Value| -> String {
        AsKebabCase(val.as_str().unwrap_or_default()).to_string()
    });

    // Keyword sanitization filter: {{ field_name | sanitize_keyword("rust") }}
    env.add_filter(
        "sanitize_keyword",
        |val: Value, lang: Option<String>| -> String {
            let name = val.as_str().unwrap_or_default();
            let target = lang.as_deref().unwrap_or("generic");
            sanitize_keyword(name, target)
        },
    );

    env
}

/// Sanitize an identifier against reserved keywords for a given target language.
pub fn sanitize_keyword(name: &str, target_language: &str) -> String {
    let keywords = match target_language.to_lowercase().as_str() {
        "rust" => rust_keywords(),
        "python" => python_keywords(),
        "cpp" | "c++" => cpp_keywords(),
        "java" => java_keywords(),
        "ts" | "typescript" | "javascript" | "js" => ts_keywords(),
        "go" => go_keywords(),
        "csharp" | "c#" | "cs" => csharp_keywords(),
        _ => generic_keywords(),
    };

    if keywords.contains(name) {
        match target_language.to_lowercase().as_str() {
            "rust" => format!("r#{}", name),
            "csharp" | "c#" | "cs" => format!("@{}", name),
            _ => format!("{}_", name),
        }
    } else {
        name.to_string()
    }
}

fn rust_keywords() -> &'static HashSet<&'static str> {
    static RUST_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    RUST_KEYWORDS.get_or_init(|| {
        [
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
            "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become",
            "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual",
            "yield", "try",
        ]
        .into_iter()
        .collect()
    })
}

fn python_keywords() -> &'static HashSet<&'static str> {
    static PY_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    PY_KEYWORDS.get_or_init(|| {
        [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield", "match", "case", "type",
        ]
        .into_iter()
        .collect()
    })
}

fn cpp_keywords() -> &'static HashSet<&'static str> {
    static CPP_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CPP_KEYWORDS.get_or_init(|| {
        [
            "alignas",
            "alignof",
            "and",
            "and_eq",
            "asm",
            "atomic_cancel",
            "atomic_commit",
            "atomic_noexcept",
            "auto",
            "bitand",
            "bitor",
            "bool",
            "break",
            "case",
            "catch",
            "char",
            "char8_t",
            "char16_t",
            "char32_t",
            "class",
            "compl",
            "concept",
            "const",
            "consteval",
            "constexpr",
            "constinit",
            "const_cast",
            "continue",
            "co_await",
            "co_return",
            "co_yield",
            "decltype",
            "default",
            "delete",
            "do",
            "double",
            "dynamic_cast",
            "else",
            "enum",
            "explicit",
            "export",
            "extern",
            "false",
            "float",
            "for",
            "friend",
            "goto",
            "if",
            "inline",
            "int",
            "long",
            "mutable",
            "namespace",
            "new",
            "noexcept",
            "not",
            "not_eq",
            "nullptr",
            "operator",
            "or",
            "or_eq",
            "private",
            "protected",
            "public",
            "reflexpr",
            "register",
            "reinterpret_cast",
            "requires",
            "return",
            "short",
            "signed",
            "sizeof",
            "static",
            "static_assert",
            "static_cast",
            "struct",
            "switch",
            "synchronized",
            "template",
            "this",
            "thread_local",
            "throw",
            "true",
            "try",
            "typedef",
            "typeid",
            "typename",
            "union",
            "unsigned",
            "using",
            "virtual",
            "void",
            "volatile",
            "wchar_t",
            "while",
            "xor",
            "xor_eq",
        ]
        .into_iter()
        .collect()
    })
}

fn java_keywords() -> &'static HashSet<&'static str> {
    static JAVA_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    JAVA_KEYWORDS.get_or_init(|| {
        [
            "abstract",
            "assert",
            "boolean",
            "break",
            "byte",
            "case",
            "catch",
            "char",
            "class",
            "const",
            "continue",
            "default",
            "do",
            "double",
            "else",
            "enum",
            "extends",
            "final",
            "finally",
            "float",
            "for",
            "goto",
            "if",
            "implements",
            "import",
            "instanceof",
            "int",
            "interface",
            "long",
            "native",
            "new",
            "package",
            "private",
            "protected",
            "public",
            "return",
            "short",
            "static",
            "strictfp",
            "super",
            "switch",
            "synchronized",
            "this",
            "throw",
            "throws",
            "transient",
            "try",
            "void",
            "volatile",
            "while",
            "record",
            "sealed",
            "permits",
            "var",
            "yield",
        ]
        .into_iter()
        .collect()
    })
}

fn ts_keywords() -> &'static HashSet<&'static str> {
    static TS_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    TS_KEYWORDS.get_or_init(|| {
        [
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "as",
            "implements",
            "interface",
            "let",
            "package",
            "private",
            "protected",
            "public",
            "static",
            "yield",
            "any",
            "boolean",
            "constructor",
            "declare",
            "get",
            "module",
            "require",
            "number",
            "set",
            "string",
            "symbol",
            "type",
            "from",
            "of",
        ]
        .into_iter()
        .collect()
    })
}

fn go_keywords() -> &'static HashSet<&'static str> {
    static GO_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    GO_KEYWORDS.get_or_init(|| {
        [
            "break",
            "case",
            "chan",
            "const",
            "continue",
            "default",
            "defer",
            "else",
            "fallthrough",
            "for",
            "func",
            "go",
            "goto",
            "if",
            "import",
            "interface",
            "map",
            "package",
            "range",
            "return",
            "select",
            "struct",
            "switch",
            "type",
            "var",
        ]
        .into_iter()
        .collect()
    })
}

fn csharp_keywords() -> &'static HashSet<&'static str> {
    static CS_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CS_KEYWORDS.get_or_init(|| {
        [
            "abstract",
            "as",
            "base",
            "bool",
            "break",
            "byte",
            "case",
            "catch",
            "char",
            "checked",
            "class",
            "const",
            "continue",
            "decimal",
            "default",
            "delegate",
            "do",
            "double",
            "else",
            "enum",
            "event",
            "explicit",
            "extern",
            "false",
            "finally",
            "fixed",
            "float",
            "for",
            "foreach",
            "goto",
            "if",
            "implicit",
            "in",
            "int",
            "interface",
            "internal",
            "is",
            "lock",
            "long",
            "namespace",
            "new",
            "null",
            "object",
            "operator",
            "out",
            "override",
            "params",
            "private",
            "protected",
            "public",
            "readonly",
            "ref",
            "return",
            "sbyte",
            "sealed",
            "short",
            "sizeof",
            "stackalloc",
            "static",
            "string",
            "struct",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "uint",
            "ulong",
            "unchecked",
            "unsafe",
            "ushort",
            "using",
            "virtual",
            "void",
            "volatile",
            "while",
            "record",
        ]
        .into_iter()
        .collect()
    })
}

fn generic_keywords() -> &'static HashSet<&'static str> {
    static GENERIC_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    GENERIC_KEYWORDS.get_or_init(|| {
        ["type", "class", "struct", "import", "export", "default"]
            .into_iter()
            .collect()
    })
}
