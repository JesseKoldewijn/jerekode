//! Anthropic Messages API provider.

use crate::error::ProviderResult;
use crate::provider::{
    CompletionChunk, CompletionRequest, CompletionResponse, ModelInfo, Provider, ProviderId,
    SharedHttpClient, env_api_key,
};
use crate::stream::parse_anthropic_sse;
use async_trait::async_trait;
use jerekode_core::MessageRole;

pub struct AnthropicProvider {
    id: ProviderId,
    base_url: String,
    api_key_env: String,
    http: SharedHttpClient,
}

impl AnthropicProvider {
    pub fn new(http: SharedHttpClient) -> Self {
        Self {
            id: ProviderId::new("anthropic"),
            base_url: "https://api.anthropic.com/v1".into(),
            api_key_env: "ANTHROPIC_API_KEY".into(),
            http,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_api_key_env(mut self, env: impl Into<String>) -> Self {
        self.api_key_env = env.into();
        self
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn display_name(&self) -> &str {
        "Anthropic"
    }

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "claude-3-5-sonnet-latest".into(),
                display_name: Some("Claude 3.5 Sonnet".into()),
                context_window: Some(200_000),
            },
            ModelInfo {
                id: "claude-3-5-haiku-latest".into(),
                display_name: Some("Claude 3.5 Haiku".into()),
                context_window: Some(200_000),
            },
        ])
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let key = env_api_key(&self.api_key_env)?;
        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));

        let mut system = None;
        let mut messages = Vec::new();
        for m in &request.messages {
            match m.role {
                MessageRole::System => system = Some(m.content.clone()),
                MessageRole::User => {
                    messages.push(serde_json::json!({"role": "user", "content": m.content}))
                }
                MessageRole::Assistant => {
                    messages.push(serde_json::json!({"role": "assistant", "content": m.content}))
                }
                MessageRole::Tool => {
                    messages.push(serde_json::json!({"role": "user", "content": m.content}))
                }
            }
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(1024),
        });
        if let Some(system) = system {
            body["system"] = serde_json::json!(system);
        }

        let response = self
            .http
            .request_json(
                "POST",
                &url,
                &[
                    ("x-api-key", key),
                    ("anthropic-version", "2023-06-01".into()),
                    ("Content-Type", "application/json".into()),
                ],
                Some(body),
            )
            .await?;

        let content = response
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let finish = response
            .get("stop_reason")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        Ok(CompletionResponse {
            content,
            model: request.model,
            finish_reason: finish,
            tool_calls: Vec::new(),
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> ProviderResult<Vec<CompletionChunk>> {
        let key = env_api_key(&self.api_key_env)?;
        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));

        let mut system = None;
        let mut messages = Vec::new();
        for m in &request.messages {
            match m.role {
                MessageRole::System => system = Some(m.content.clone()),
                MessageRole::User => {
                    messages.push(serde_json::json!({"role": "user", "content": m.content}))
                }
                MessageRole::Assistant => {
                    messages.push(serde_json::json!({"role": "assistant", "content": m.content}))
                }
                MessageRole::Tool => {
                    messages.push(serde_json::json!({"role": "user", "content": m.content}))
                }
            }
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "stream": true,
        });
        if let Some(system) = system {
            body["system"] = serde_json::json!(system);
        }

        let text = self
            .http
            .request_text(
                "POST",
                &url,
                &[
                    ("x-api-key", key),
                    ("anthropic-version", "2023-06-01".into()),
                    ("Content-Type", "application/json".into()),
                    ("Accept", "text/event-stream".into()),
                ],
                Some(body),
            )
            .await?;
        parse_anthropic_sse(&text, &request.model)
    }

    async fn health_check(&self) -> ProviderResult<()> {
        let _ = env_api_key(&self.api_key_env)?;
        Ok(())
    }
}
