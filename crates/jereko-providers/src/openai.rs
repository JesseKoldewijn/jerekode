//! OpenAI-compatible Chat Completions provider.

use crate::error::ProviderResult;
use crate::provider::{
    env_api_key, CompletionRequest, CompletionResponse, ModelInfo, Provider, ProviderId,
    SharedHttpClient,
};
use async_trait::async_trait;
use jereko_core::MessageRole;

pub struct OpenAiProvider {
    id: ProviderId,
    base_url: String,
    api_key_env: String,
    http: SharedHttpClient,
}

impl OpenAiProvider {
    pub fn new(http: SharedHttpClient) -> Self {
        Self {
            id: ProviderId::new("openai"),
            base_url: "https://api.openai.com/v1".into(),
            api_key_env: "OPENAI_API_KEY".into(),
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
impl Provider for OpenAiProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn display_name(&self) -> &str {
        "OpenAI"
    }

    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let key = env_api_key(&self.api_key_env)?;
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let body = self
            .http
            .request_json(
                "GET",
                &url,
                &[("Authorization", format!("Bearer {key}"))],
                None,
            )
            .await?;
        let models = body
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| {
                let id = m.get("id")?.as_str()?.to_string();
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
        let key = env_api_key(&self.api_key_env)?;
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
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
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });
        if let Some(max) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max);
        }
        let response = self
            .http
            .request_json(
                "POST",
                &url,
                &[
                    ("Authorization", format!("Bearer {key}")),
                    ("Content-Type", "application/json".into()),
                ],
                Some(body),
            )
            .await?;
        let content = response
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let finish = response
            .pointer("/choices/0/finish_reason")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        Ok(CompletionResponse {
            content,
            model: request.model,
            finish_reason: finish,
        })
    }

    async fn health_check(&self) -> ProviderResult<()> {
        let _ = env_api_key(&self.api_key_env)?;
        Ok(())
    }
}
