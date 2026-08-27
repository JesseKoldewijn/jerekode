//! Ollama local HTTP provider.

use crate::error::ProviderResult;
use crate::provider::{
    CompletionChunk, CompletionRequest, CompletionResponse, ModelInfo, Provider, ProviderId,
    SharedHttpClient,
};
use crate::stream::parse_ollama_ndjson;
use async_trait::async_trait;
use jerekode_core::MessageRole;

pub struct OllamaProvider {
    id: ProviderId,
    base_url: String,
    http: SharedHttpClient,
}

impl OllamaProvider {
    pub fn new(http: SharedHttpClient) -> Self {
        Self {
            id: ProviderId::new("ollama"),
            base_url: std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
            http,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn display_name(&self) -> &str {
        "Ollama"
    }

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let body = self.http.request_json("GET", &url, &[], None).await?;
        let models = body
            .get("models")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| {
                let id = m.get("name")?.as_str()?.to_string();
                Some(ModelInfo {
                    id,
                    display_name: None,
                    context_window: None,
                })
            })
            .collect();
        Ok(models)
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                };
                serde_json::json!({"role": role, "content": m.content})
            })
            .collect();
        let body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": false,
        });
        let response = self
            .http
            .request_json(
                "POST",
                &url,
                &[("Content-Type", "application/json".into())],
                Some(body),
            )
            .await?;
        let content = response
            .pointer("/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(CompletionResponse {
            content,
            model: request.model,
            finish_reason: Some("stop".into()),
            tool_calls: Vec::new(),
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> ProviderResult<Vec<CompletionChunk>> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                };
                serde_json::json!({"role": role, "content": m.content})
            })
            .collect();
        let body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
        });
        let text = self
            .http
            .request_text(
                "POST",
                &url,
                &[("Content-Type", "application/json".into())],
                Some(body),
            )
            .await?;
        parse_ollama_ndjson(&text, &request.model)
    }

    async fn health_check(&self) -> ProviderResult<()> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let _ = self.http.request_json("GET", &url, &[], None).await?;
        Ok(())
    }
}
