use thiserror::Error;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {path}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config: {path}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("config validation failed: {0}")]
    Validation(String),

    #[error("invalid environment variable {var}: {value}")]
    InvalidEnv { var: String, value: String },
}
