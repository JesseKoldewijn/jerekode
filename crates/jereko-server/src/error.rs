use thiserror::Error;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("failed to bind {0}: {1}")]
    Bind(String, std::io::Error),

    #[error("server error: {0}")]
    Serve(String),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("internal error: {0}")]
    Internal(String),
}
