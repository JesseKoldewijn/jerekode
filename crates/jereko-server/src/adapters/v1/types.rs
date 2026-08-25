use serde::{Deserialize, Serialize};

/// v1 wire-format types (subset stub).
#[derive(Debug, Clone, Deserialize)]
pub struct V1CreateSessionRequest {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1CreateSessionResponse {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct V1SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1SendMessageResponse {
    pub content: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1ListProvidersResponse {
    pub providers: Vec<V1ProviderSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1ProviderSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1ErrorResponse {
    pub error: String,
}
