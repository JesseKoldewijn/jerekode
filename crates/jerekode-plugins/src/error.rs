use thiserror::Error;

pub type PluginResult<T> = Result<T, PluginError>;

/// Clear runtime/build hint when Bun/TS plugins are requested without Bun sidecar support.
pub const BUN_SIDECAR_UNAVAILABLE_MSG: &str = concat!(
    "this build was compiled without Bun sidecar support (native-only / --no-default-features). ",
    "Download the full build from GitHub Releases, or use only native/wasm plugins in config."
);

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),

    #[error("host error ({host}): {message}")]
    Host { host: String, message: String },

    #[error("sidecar IPC error: {0}")]
    Sidecar(String),

    #[error("native plugin error: {0}")]
    Native(String),

    #[error("orchestrator error: {0}")]
    Orchestrator(String),
}
