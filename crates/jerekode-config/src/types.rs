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

    /// Plugin configuration (resolved by PluginOrchestrator).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginEntry>,

    /// Optional SQLite path for durable session storage.
    /// When unset, the server uses an in-memory session store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_db: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

/// Plugin entry in config — unqualified string (Bun) or explicit host object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PluginEntry {
    /// Bun plugin (default host): `"@acme/server-plugin"`.
    Bun(String),
    /// Explicit native dylib path.
    Native { native: String },
    /// Explicit WASM path (Phase 4).
    Wasm { wasm: String },
    /// Legacy structured form with name/path.
    Named {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
}

impl PluginEntry {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Bun(s) | Self::Named { name: s, .. } => s,
            Self::Native { native } | Self::Wasm { wasm: native } => native,
        }
    }
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
