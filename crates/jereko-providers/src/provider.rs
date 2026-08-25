use crate::error::{ProviderError, ProviderResult};
use async_trait::async_trait;
use jereko_core::Message;
use serde::{Deserialize, Serialize};
use std::fmt;

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

/// Trait implemented by each LLM provider adapter.
///
/// Designed for 75+ providers: keep implementations thin; shared HTTP/auth
/// utilities will live in submodules as the registry grows.
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &ProviderId;

    fn display_name(&self) -> &str;

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>>;

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse>;

    /// Optional health check for provider availability.
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

/// Resolve a provider from a registry by id string.
#[allow(dead_code)] // TODO(phase-1): wire into server handlers
pub fn resolve<'a>(
    registry: &'a crate::registry::ProviderRegistry,
    id: &str,
) -> ProviderResult<&'a dyn Provider> {
    registry
        .get(id)
        .ok_or_else(|| ProviderError::NotFound(id.into()))
}
