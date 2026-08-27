use jerekode_core::Session;
use serde::{Deserialize, Serialize};

/// v2 wire-format types (subset stub).
#[derive(Debug, Clone, Deserialize)]
pub struct V2CreateSessionRequest {
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V2CreateSessionResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Deserialize)]
pub struct V2SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct V2SendMessageResponse {
    pub session: Session,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct V2ErrorResponse {
    pub code: String,
    pub message: String,
}
