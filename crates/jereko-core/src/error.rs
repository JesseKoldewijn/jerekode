use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("invalid session state: {0}")]
    InvalidSessionState(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
