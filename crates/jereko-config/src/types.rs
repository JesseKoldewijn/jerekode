use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal stub for `opencode.json` shape.
///
/// Fields will expand in Phase 1 to match OpenCode config schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeConfig {
    /// Default provider id (e.g. `"anthropic"`, `"openai"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Default model id for the active provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// HTTP server bind address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// HTTP server port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Per-provider overrides keyed by provider id.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub providers: HashMap<String, ProviderOverride>,

    /// Plugin configuration (resolved by the Bun sidecar).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginRef>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Minimal stub for `tui.json` shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keymap: Option<String>,

    /// Sidecar process settings (Bun plugin host).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidecar: Option<SidecarConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SidecarConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_path: Option<String>,
}
