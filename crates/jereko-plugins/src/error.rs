use thiserror::Error;

pub type PluginResult<T> = Result<T, PluginError>;

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
