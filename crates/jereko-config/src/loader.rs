use crate::error::{ConfigError, ConfigResult};
use crate::types::{OpenCodeConfig, TuiConfig};
use std::fs;
use std::path::{Path, PathBuf};

/// Identifies which precedence layer a config fragment came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLayer {
    Default,
    Global,
    Project,
    Environment,
    Cli,
}

/// How two config layers combine during merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Replace the entire target with the source (scalars).
    Replace,
    /// Deep-merge maps; source keys override target keys.
    DeepMerge,
}

/// Loads and merges configuration from multiple precedence layers.
#[derive(Debug, Default)]
pub struct ConfigLoader {
    opencode: OpenCodeConfig,
    tui: TuiConfig,
    loaded_layers: Vec<ConfigLayer>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            opencode: OpenCodeConfig::default(),
            tui: TuiConfig::default(),
            loaded_layers: vec![ConfigLayer::Default],
        }
    }

    /// Parse a JSON/JSONC config file from disk.
    ///
    /// TODO(phase-1): swap `serde_json` for a JSONC parser (comments, trailing commas).
    pub fn load_file(&mut self, path: impl AsRef<Path>, layer: ConfigLayer) -> ConfigResult<()> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
            path: path.display().to_string(),
            source,
        })?;

        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains("tui"))
        {
            let parsed: TuiConfig =
                serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            self.tui = merge_tui(&self.tui, &parsed);
        } else {
            let parsed: OpenCodeConfig =
                serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            self.opencode = merge_opencode(&self.opencode, &parsed);
        }

        self.loaded_layers.push(layer);
        Ok(())
    }

    pub fn opencode(&self) -> &OpenCodeConfig {
        &self.opencode
    }

    pub fn tui(&self) -> &TuiConfig {
        &self.tui
    }

    pub fn loaded_layers(&self) -> &[ConfigLayer] {
        &self.loaded_layers
    }

    /// Resolve standard config search paths (stub — returns empty if missing).
    pub fn discover_paths(project_root: impl AsRef<Path>) -> ConfigPaths {
        ConfigPaths {
            global_opencode: default_global_config("opencode.json"),
            global_tui: default_global_config("tui.json"),
            project_opencode: project_root
                .as_ref()
                .join(".opencode")
                .join("opencode.json"),
            project_tui: project_root.as_ref().join(".opencode").join("tui.json"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub global_opencode: PathBuf,
    pub global_tui: PathBuf,
    pub project_opencode: PathBuf,
    pub project_tui: PathBuf,
}

fn default_global_config(filename: &str) -> PathBuf {
    dirs_fallback().join("opencode").join(filename)
}

fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".config"))
        .join(".config")
}

fn merge_opencode(base: &OpenCodeConfig, overlay: &OpenCodeConfig) -> OpenCodeConfig {
    let mut merged = base.clone();

    if overlay.provider.is_some() {
        merged.provider = overlay.provider.clone();
    }
    if overlay.model.is_some() {
        merged.model = overlay.model.clone();
    }
    if overlay.host.is_some() {
        merged.host = overlay.host.clone();
    }
    if overlay.port.is_some() {
        merged.port = overlay.port;
    }

    for (key, value) in &overlay.providers {
        merged.providers.insert(key.clone(), value.clone());
    }

    if !overlay.plugins.is_empty() {
        merged.plugins = overlay.plugins.clone();
    }

    merged
}

fn merge_tui(base: &TuiConfig, overlay: &TuiConfig) -> TuiConfig {
    let mut merged = base.clone();

    if overlay.theme.is_some() {
        merged.theme = overlay.theme.clone();
    }
    if overlay.keymap.is_some() {
        merged.keymap = overlay.keymap.clone();
    }
    if overlay.sidecar.is_some() {
        merged.sidecar = overlay.sidecar.clone();
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn later_layer_overrides_scalar() {
        let base = OpenCodeConfig {
            provider: Some("openai".into()),
            model: Some("gpt-4".into()),
            ..Default::default()
        };
        let overlay = OpenCodeConfig {
            model: Some("gpt-4o".into()),
            ..Default::default()
        };

        let merged = merge_opencode(&base, &overlay);
        assert_eq!(merged.provider.as_deref(), Some("openai"));
        assert_eq!(merged.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn precedence_order_is_documented() {
        assert!(ConfigLayer::Default < ConfigLayer::Global);
        assert!(ConfigLayer::Global < ConfigLayer::Project);
        assert!(ConfigLayer::Project < ConfigLayer::Environment);
        assert!(ConfigLayer::Environment < ConfigLayer::Cli);
    }
}
