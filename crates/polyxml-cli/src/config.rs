use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error reading configuration: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML syntax error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Invalid glob pattern '{pattern}': {error}")]
    GlobPattern {
        pattern: String,
        error: glob::PatternError,
    },

    #[error("Failed to read glob path: {0}")]
    Glob(#[from] glob::GlobError),
}

/// The top-level `polyxml.toml` workspace manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub workspace: Option<WorkspaceSection>,
    #[serde(default)]
    pub generate: Vec<TargetConfig>,
    pub codegen: Option<HashMap<String, CodegenTargetConfig>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSection {
    pub name: Option<String>,
    #[serde(default)]
    pub schemas: Vec<String>,
    pub include_dirs: Option<Vec<String>>,
    pub output_base_dir: Option<String>,
    pub custom_header: Option<String>,
}

/// Target configuration from either `[[generate]]` or `[codegen.<target>]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetConfig {
    pub target: String,
    pub output: String,
    pub enabled: Option<bool>,
    pub backend: Option<String>,
    pub package: Option<String>,
    pub namespace: Option<String>,
    pub strict_facets: Option<bool>,
    pub slots: Option<bool>,
    pub kw_only: Option<bool>,
    pub zero_copy: Option<bool>,
    pub codecs: Option<bool>,
    pub standard: Option<String>,
    pub derive_traits: Option<Vec<String>>,
    pub box_cycles: Option<bool>,
    pub modules: Option<bool>,
    pub mode: Option<String>,
    pub serializer: Option<String>,
    pub zod: Option<bool>,
    pub source_gen: Option<bool>,
    pub record_kind: Option<String>,
    pub style: Option<String>,
    pub builder: Option<bool>,
    pub codec: Option<String>,
    pub rkyv: Option<bool>,
    pub custom_header: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodegenTargetConfig {
    pub enabled: Option<bool>,
    pub output: Option<String>,
    pub backend: Option<String>,
    pub package: Option<String>,
    pub namespace: Option<String>,
    pub strict_facets: Option<bool>,
    pub slots: Option<bool>,
    pub kw_only: Option<bool>,
    pub zero_copy: Option<bool>,
    pub codecs: Option<bool>,
    pub standard: Option<String>,
    pub derive_traits: Option<Vec<String>>,
    pub box_cycles: Option<bool>,
    pub modules: Option<bool>,
    pub mode: Option<String>,
    pub serializer: Option<String>,
    pub zod: Option<bool>,
    pub source_gen: Option<bool>,
    pub record_kind: Option<String>,
    pub style: Option<String>,
    pub builder: Option<bool>,
    pub codec: Option<String>,
    pub rkyv: Option<bool>,
    pub custom_header: Option<String>,
}

impl std::str::FromStr for WorkspaceManifest {
    type Err = ConfigError;

    fn from_str(toml_str: &str) -> Result<Self, Self::Err> {
        let manifest: WorkspaceManifest = toml::from_str(toml_str)?;
        Ok(manifest)
    }
}

impl WorkspaceManifest {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        content.parse()
    }

    /// Retrieve all configured target configurations, combining `[[generate]]`
    /// and `[codegen.<target>]` definitions.
    pub fn resolved_targets(&self) -> Vec<TargetConfig> {
        let mut targets = Vec::new();

        let ws_header = self
            .workspace
            .as_ref()
            .and_then(|w| w.custom_header.clone());

        // 1. Array of tables [[generate]]
        for gen in &self.generate {
            if gen.enabled.unwrap_or(true) {
                let mut target = gen.clone();
                if target.custom_header.is_none() {
                    target.custom_header = ws_header.clone();
                }
                targets.push(target);
            }
        }

        // 2. Table-based [codegen.<lang>]
        if let Some(ref codegen_map) = self.codegen {
            for (lang, cfg) in codegen_map {
                if cfg.enabled.unwrap_or(true) {
                    let output = cfg
                        .output
                        .clone()
                        .unwrap_or_else(|| format!("generated/{}", lang));

                    targets.push(TargetConfig {
                        target: lang.clone(),
                        output,
                        enabled: cfg.enabled,
                        backend: cfg.backend.clone(),
                        package: cfg.package.clone(),
                        namespace: cfg.namespace.clone(),
                        strict_facets: cfg.strict_facets,
                        slots: cfg.slots,
                        kw_only: cfg.kw_only,
                        zero_copy: cfg.zero_copy,
                        codecs: cfg.codecs,
                        standard: cfg.standard.clone(),
                        derive_traits: cfg.derive_traits.clone(),
                        box_cycles: cfg.box_cycles,
                        modules: cfg.modules,
                        mode: cfg.mode.clone(),
                        serializer: cfg.serializer.clone(),
                        zod: cfg.zod,
                        source_gen: cfg.source_gen,
                        record_kind: cfg.record_kind.clone(),
                        style: cfg.style.clone(),
                        builder: cfg.builder,
                        codec: cfg.codec.clone(),
                        rkyv: cfg.rkyv,
                        custom_header: cfg.custom_header.clone().or_else(|| ws_header.clone()),
                    });
                }
            }
        }

        targets
    }

    /// Expand all schema glob patterns in `workspace.schemas` relative to base directory.
    pub fn expand_schemas(&self, base_dir: &Path) -> Result<Vec<PathBuf>, ConfigError> {
        let mut paths = Vec::new();

        let Some(ref ws) = self.workspace else {
            return Ok(paths);
        };

        for pattern in &ws.schemas {
            let full_pattern = if Path::new(pattern).is_absolute() {
                pattern.clone()
            } else {
                base_dir.join(pattern).to_string_lossy().to_string()
            };

            let entries = glob(&full_pattern).map_err(|e| ConfigError::GlobPattern {
                pattern: full_pattern.clone(),
                error: e,
            })?;

            for entry in entries {
                let path = entry?;
                if path.is_file() {
                    paths.push(path);
                }
            }
        }

        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
