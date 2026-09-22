//! Shared validation and normalization for direct generation and both manifest forms.
use super::*;
use std::io::{Error, ErrorKind, Result};

fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidInput, message.into())
}

pub(crate) fn target_options(target: &config::TargetConfig) -> TargetEmitOptions<'_> {
    TargetEmitOptions {
        backend: target.backend.as_deref(),
        features: &target.features,
        slots: target.slots,
        kw_only: target.kw_only,
        package: target.package.as_deref().or(target.namespace.as_deref()),
        mode: target
            .mode
            .as_deref()
            .or_else(|| target.modules.filter(|v| *v).map(|_| "modules")),
        zero_copy: target.zero_copy,
        codecs: target.codecs,
        zod: target.zod,
        source_gen: target.source_gen,
        record_kind: target.record_kind.as_deref(),
        style: target.style.as_deref(),
        builder: target.builder,
        codec: target.codec.as_deref(),
        rkyv: target.rkyv,
        custom_header: target.custom_header.as_deref(),
    }
}

impl<'a> TargetEmitOptions<'a> {
    pub(crate) fn resolve(mut self, lang: &str, warn: bool) -> Result<Self> {
        let language = lang.to_lowercase();
        let target = match language.as_str() {
            "python" | "py" => "python",
            "rust" | "rs" => "rust",
            "typescript" | "ts" => "typescript",
            "c++" | "cpp" => "cpp",
            "cs" | "c#" | "csharp" => "csharp",
            "java" => "java",
            "go" => "go",
            _ => return Err(invalid(format!("Unknown target '{lang}'. Supported targets: python, rust, cpp, java, typescript, go, csharp."))),
        };
        let supported_backends = match target {
            "python" => "dataclass, pydantic",
            "typescript" => "interfaces, zod, valibot, typebox",
            "java" => "standard, jackson",
            "csharp" => "standard, source-gen",
            "cpp" => "standard, glaze",
            "go" => "standard, easyjson, sonic",
            _ => "standard",
        };
        if let Some(backend) = self.backend {
            let valid = match target {
                "python" => PythonBackend::from_str_loose(backend).is_some(),
                "typescript" => TypeScriptBackend::from_str_loose(backend).is_some(),
                "java" => JavaBackend::from_str_loose(backend).is_some(),
                "cpp" => CppBackend::from_str_loose(backend).is_some(),
                "go" => GoBackend::from_str_loose(backend).is_some(),
                "csharp" => matches!(backend, "standard" | "source-gen"),
                _ => backend == "standard",
            };
            if !valid {
                return Err(invalid(format!("backend '{backend}' is not supported for target '{target}'. Supported backends: {supported_backends}.")));
            }
        }

        // Reject even explicit false legacy flags on unrelated targets: they are
        // almost always mistakes in scripts, and must not silently do nothing.
        for (name, present, required, replacement) in [
            ("zod", self.zod.is_some(), "typescript", "--backend zod"),
            (
                "source-gen",
                self.source_gen.is_some(),
                "csharp",
                "--backend source-gen",
            ),
            (
                "zero-copy",
                self.zero_copy.is_some(),
                "rust",
                "--feature zero-copy",
            ),
            ("rkyv", self.rkyv.is_some(), "rust", "--feature rkyv"),
            (
                "builder",
                self.builder.is_some(),
                "java",
                "--feature builder",
            ),
            (
                "codec",
                self.codec.is_some(),
                "java",
                "--feature direct-codec (or omit for annotations)",
            ),
            (
                "record-kind",
                self.record_kind.is_some(),
                "csharp",
                "--style record-class or --style record-struct",
            ),
        ] {
            if present {
                if target != required {
                    return Err(invalid(format!("option '{name}' is not supported for target '{target}'; use it with '{required}'.")));
                }
                if warn {
                    eprintln!("warning: option '--{name}' is deprecated. Use '{replacement}' instead (manifest: backend/style/features).");
                }
            }
        }
        if self.zod == Some(true) {
            if self.backend.is_some_and(|v| {
                TypeScriptBackend::from_str_loose(v) != Some(TypeScriptBackend::Zod)
            }) {
                return Err(invalid(
                    "--zod conflicts with the selected backend; use --backend zod.",
                ));
            }
            self.backend = Some("zod");
        } else if self.zod == Some(false)
            && self.backend.is_some_and(|v| {
                TypeScriptBackend::from_str_loose(v) == Some(TypeScriptBackend::Zod)
            })
        {
            return Err(invalid("--zod false conflicts with --backend zod."));
        }
        if target == "csharp" {
            if let Some(backend) = self.backend {
                let enabled = backend == "source-gen";
                if self.source_gen.is_some_and(|value| value != enabled) {
                    return Err(invalid("source-gen conflicts with the selected backend."));
                }
                self.source_gen = Some(enabled);
            }
        }
        let supported_styles: &[&str] = match target {
            "java" => &["record", "pojo", "class"],
            "csharp" => &["record", "pojo", "class", "record-class", "record-struct"],
            "python" => &["dataclass"],
            _ => &[],
        };
        if let Some(style) = self.style {
            if !supported_styles.contains(&style) {
                return Err(invalid(format!(
                    "style '{style}' is not supported for target '{target}'. Supported styles: {}.",
                    if supported_styles.is_empty() {
                        "none".into()
                    } else {
                        supported_styles.join(", ")
                    }
                )));
            }
            if target == "python"
                && self.backend.is_some_and(|v| {
                    PythonBackend::from_str_loose(v) == Some(PythonBackend::Pydantic)
                })
            {
                return Err(invalid(
                    "style 'dataclass' requires the Python dataclass backend.",
                ));
            }
        }
        if let Some(kind) = self.record_kind {
            if CSharpRecordKind::from_str_loose(kind).is_none() {
                return Err(invalid(format!(
                    "Unknown record-kind '{kind}'; use class or struct."
                )));
            }
        }
        if target == "csharp" {
            let kind = match self.style {
                Some("record-class" | "class" | "pojo") => Some("class"),
                Some("record-struct") => Some("struct"),
                _ => None,
            };
            if let Some(kind) = kind {
                if self.record_kind.is_some_and(|v| {
                    CSharpRecordKind::from_str_loose(v) != CSharpRecordKind::from_str_loose(kind)
                }) {
                    return Err(invalid("style conflicts with record-kind."));
                }
                self.record_kind = Some(kind);
            }
        }
        if self
            .codec
            .is_some_and(|v| !matches!(v, "annotation" | "direct"))
        {
            return Err(invalid("Unknown codec; use annotation or direct."));
        }
        if let Some(mode) = self.mode {
            if target != "cpp" || CppMode::from_str_loose(mode).is_none() {
                return Err(invalid(format!("mode '{mode}' is not supported for target '{target}'; C++ supports header or modules.")));
            }
        }
        if target != "python" && (self.slots.is_some() || self.kw_only.is_some()) {
            return Err(invalid("slots and kw_only are Python-only options."));
        }
        let supported_features: &[&str] = match target {
            "rust" => &["zero-copy", "rkyv"],
            "java" => &["builder", "direct-codec"],
            "python" => &["slots", "kw-only"],
            _ => &[],
        };
        for feature in self.features {
            if !supported_features.contains(&feature.as_str()) {
                return Err(invalid(format!("feature '{feature}' is not supported for target '{target}'. Supported features: {}.", if supported_features.is_empty() { "none".into() } else { supported_features.join(", ") })));
            }
            let option = match feature.as_str() {
                "zero-copy" => &mut self.zero_copy,
                "rkyv" => &mut self.rkyv,
                "builder" => &mut self.builder,
                "slots" => &mut self.slots,
                "kw-only" => &mut self.kw_only,
                "direct-codec" => {
                    if self.codec == Some("annotation") {
                        return Err(invalid(
                            "feature 'direct-codec' conflicts with codec 'annotation'.",
                        ));
                    }
                    self.codec = Some("direct");
                    continue;
                }
                _ => unreachable!(),
            };
            if *option == Some(false) {
                return Err(invalid(format!(
                    "feature '{feature}' conflicts with its explicitly disabled legacy option."
                )));
            }
            *option = Some(true);
        }
        if target == "python"
            && self
                .backend
                .is_some_and(|v| PythonBackend::from_str_loose(v) == Some(PythonBackend::Pydantic))
            && (self.slots.is_some() || self.kw_only.is_some())
        {
            return Err(invalid(
                "slots and kw-only require the Python dataclass backend.",
            ));
        }
        Ok(self)
    }
}
