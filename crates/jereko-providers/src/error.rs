use thiserror::Error;

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider not found: {0}")]
    NotFound(String),

    #[error("provider already registered: {0}")]
    AlreadyRegistered(String),

    #[error("provider error ({provider}): {message}")]
    ProviderFailure { provider: String, message: String },

    #[error("model not found: {provider}/{model}")]
    ModelNotFound { provider: String, model: String },
}
