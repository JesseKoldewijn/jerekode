use crate::error::{ProviderError, ProviderResult};
use async_trait::async_trait;
use jerekode_core::Message;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Stable provider identifier (e.g. `"anthropic"`, `"openai"`, `"ollama"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: Option<String>,
    pub context_window: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// One incremental piece of a streaming completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionChunk {
    pub delta: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    pub model: String,
}

/// Trait implemented by each LLM provider adapter.
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &ProviderId;

    fn display_name(&self) -> &str;

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>>;

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse>;

    /// Streaming completion seam. Default wraps [`Self::complete`] as a single chunk.
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> ProviderResult<Vec<CompletionChunk>> {
        let response = self.complete(request).await?;
        Ok(vec![CompletionChunk {
            delta: response.content,
            finish_reason: response.finish_reason,
            model: response.model,
        }])
    }

    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}

/// Stub provider for workspace scaffolding and tests.
pub struct StubProvider {
    id: ProviderId,
}

impl StubProvider {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ProviderId::new(id),
        }
    }
}

#[async_trait]
impl Provider for StubProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn display_name(&self) -> &str {
        "Stub Provider"
    }

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: "stub-model".into(),
            display_name: Some("Stub Model".into()),
            context_window: Some(8192),
        }])
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        Ok(CompletionResponse {
            content: format!(
                "[stub:{}] received {} messages",
                self.id,
                request.messages.len()
            ),
            model: request.model,
            finish_reason: Some("stop".into()),
        })
    }
}

/// Shared HTTP client seam for provider adapters (mockable in tests).
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn request_json(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> ProviderResult<serde_json::Value>;

    /// Raw response body (SSE / NDJSON streams).
    async fn request_text(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> ProviderResult<String>;
}

/// Default reqwest-backed HTTP client.
#[derive(Clone, Default)]
pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl ReqwestHttpClient {
    async fn send_raw(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> ProviderResult<(reqwest::StatusCode, String)> {
        let mut req = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            other => {
                return Err(ProviderError::ProviderFailure {
                    provider: "http".into(),
                    message: format!("unsupported method {other}"),
                });
            }
        };
        for (k, v) in headers {
            req = req.header(*k, v);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::ProviderFailure {
                provider: "http".into(),
                message: e.to_string(),
            })?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| ProviderError::ProviderFailure {
                provider: "http".into(),
                message: e.to_string(),
            })?;
        Ok((status, text))
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn request_json(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> ProviderResult<serde_json::Value> {
        let (status, text) = self.send_raw(method, url, headers, body).await?;
        if !status.is_success() {
            return Err(ProviderError::ProviderFailure {
                provider: "http".into(),
                message: format!("HTTP {status}: {text}"),
            });
        }
        serde_json::from_str(&text).map_err(|e| ProviderError::ProviderFailure {
            provider: "http".into(),
            message: format!("invalid JSON: {e}"),
        })
    }

    async fn request_text(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> ProviderResult<String> {
        let (status, text) = self.send_raw(method, url, headers, body).await?;
        if !status.is_success() {
            return Err(ProviderError::ProviderFailure {
                provider: "http".into(),
                message: format!("HTTP {status}: {text}"),
            });
        }
        Ok(text)
    }
}

pub type SharedHttpClient = Arc<dyn HttpClient>;

/// Resolve a provider from a registry by id string.
pub fn resolve<'a>(
    registry: &'a crate::registry::ProviderRegistry,
    id: &str,
) -> ProviderResult<&'a dyn Provider> {
    registry
        .get(id)
        .ok_or_else(|| ProviderError::NotFound(id.into()))
}

pub fn env_api_key(var: &str) -> ProviderResult<String> {
    std::env::var(var).map_err(|_| ProviderError::ProviderFailure {
        provider: var.into(),
        message: format!("missing API key env var {var}"),
    })
}
