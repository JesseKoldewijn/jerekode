use crate::error::{ConfigError, ConfigResult};
use crate::jsonc::parse_jsonc;
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

/// CLI-level overrides applied after all file/env layers.
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
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

    /// Load all discovered config paths with full precedence:
    /// defaults → global → project → env → CLI.
    ///
    /// Global config parse failures are logged and skipped so a broken user-level
    /// file does not prevent project config from loading.
    pub fn load_discovered(
        project_root: impl AsRef<Path>,
        cli: &CliOverrides,
    ) -> ConfigResult<Self> {
        let mut loader = Self::new();
        let paths = Self::discover_paths(&project_root);

        for (path, layer) in [
            (&paths.global_opencode, ConfigLayer::Global),
            (&paths.global_tui, ConfigLayer::Global),
            (&paths.project_opencode, ConfigLayer::Project),
            (&paths.project_tui, ConfigLayer::Project),
        ] {
            if path.exists() {
                if let Err(err) = loader.load_file(path, layer) {
                    if layer == ConfigLayer::Global {
                        tracing::warn!(path = %path.display(), %err, "skipping invalid global config");
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        loader.apply_env_overrides()?;
        loader.apply_cli_overrides(cli);
        Ok(loader)
    }

    /// Parse a JSONC config file from disk.
    pub fn load_file(&mut self, path: impl AsRef<Path>, layer: ConfigLayer) -> ConfigResult<()> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
            path: path.display().to_string(),
            source,
        })?;
        let display = path.display().to_string();

        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains("tui"))
        {
            let parsed: TuiConfig = parse_jsonc(&raw, &display)?;
            self.tui = merge_tui(&self.tui, &parsed);
        } else {
            let parsed: OpenCodeConfig = parse_jsonc(&raw, &display)?;
            self.opencode = merge_opencode(&self.opencode, &parsed);
        }

        self.loaded_layers.push(layer);
        Ok(())
    }

    pub fn apply_env_overrides(&mut self) -> ConfigResult<()> {
        let mut applied = false;

        if let Some(provider) = env_var(&["JEREKO_PROVIDER", "OPENCODE_PROVIDER"]) {
            self.opencode.provider = Some(provider);
            applied = true;
        }
        if let Some(model) = env_var(&["JEREKO_MODEL", "OPENCODE_MODEL"]) {
            self.opencode.model = Some(model);
            applied = true;
        }
        if let Some(host) = env_var(&["JEREKO_HOST", "OPENCODE_HOST"]) {
            self.opencode.host = Some(host);
            applied = true;
        }
        if let Some(port_str) = env_var(&["JEREKO_PORT", "OPENCODE_PORT"]) {
            let port: u16 = port_str.parse().map_err(|_| ConfigError::InvalidEnv {
                var: "JEREKO_PORT/OPENCODE_PORT".into(),
                value: port_str,
            })?;
            self.opencode.port = Some(port);
            applied = true;
        }

        if applied {
            self.loaded_layers.push(ConfigLayer::Environment);
        }
        Ok(())
    }

    pub fn apply_cli_overrides(&mut self, cli: &CliOverrides) {
        let mut applied = false;

        if let Some(ref provider) = cli.provider {
            self.opencode.provider = Some(provider.clone());
            applied = true;
        }
        if let Some(ref model) = cli.model {
            self.opencode.model = Some(model.clone());
            applied = true;
        }
        if let Some(ref host) = cli.host {
            self.opencode.host = Some(host.clone());
            applied = true;
        }
        if let Some(port) = cli.port {
            self.opencode.port = Some(port);
            applied = true;
        }

        if applied {
            self.loaded_layers.push(ConfigLayer::Cli);
        }
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

    /// Resolve standard config search paths.
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

fn env_var(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .filter(|v| !v.is_empty())
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
    use crate::types::OpenCodeConfig;
    use std::io::Write;

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

    #[test]
    fn project_overrides_global() {
        let dir = tempfile::tempdir().unwrap();
        let global = dir.path().join("global.json");
        let project = dir.path().join(".opencode/opencode.json");
        fs::create_dir_all(project.parent().unwrap()).unwrap();

        fs::write(
            &global,
            r#"{"provider":"openai","model":"gpt-4o","port":4096}"#,
        )
        .unwrap();
        fs::write(&project, r#"{"model":"gpt-4o-mini","port":8080}"#).unwrap();

        let mut loader = ConfigLoader::new();
        loader.load_file(&global, ConfigLayer::Global).unwrap();
        loader.load_file(&project, ConfigLayer::Project).unwrap();

        assert_eq!(loader.opencode().provider.as_deref(), Some("openai"));
        assert_eq!(loader.opencode().model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(loader.opencode().port, Some(8080));
    }

    #[test]
    fn cli_overrides_env() {
        let mut loader = ConfigLoader::new();
        loader.opencode.provider = Some("openai".into());
        loader.loaded_layers.push(ConfigLayer::Environment);

        loader.apply_cli_overrides(&CliOverrides {
            provider: Some("anthropic".into()),
            ..Default::default()
        });

        assert_eq!(loader.opencode().provider.as_deref(), Some("anthropic"));
        assert!(loader.loaded_layers().contains(&ConfigLayer::Cli));
    }

    #[test]
    fn loads_jsonc_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("opencode.jsonc");
        let mut file = fs::File::create(&path).unwrap();
        write!(file, r#"{{"provider":"anthropic","port":4096,}}"#).unwrap();

        let mut loader = ConfigLoader::new();
        loader.load_file(&path, ConfigLayer::Project).unwrap();
        assert_eq!(loader.opencode().provider.as_deref(), Some("anthropic"));
    }
}
