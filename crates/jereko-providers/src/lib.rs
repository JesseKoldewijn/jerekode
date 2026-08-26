//! Provider registry designed for 75+ providers with plugin-ready extension points.

mod anthropic;
mod compat;
mod error;
mod ollama;
mod openai;
mod provider;
mod registry;
mod stream;

pub use anthropic::AnthropicProvider;
pub use compat::{groq_provider, openrouter_provider};
pub use error::{ProviderError, ProviderResult};
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use provider::{
    CompletionChunk, CompletionRequest, CompletionResponse, HttpClient, ModelInfo, Provider,
    ProviderId, ReqwestHttpClient, SharedHttpClient, StubProvider, env_api_key, resolve,
};
pub use registry::ProviderRegistry;
pub use stream::{parse_anthropic_sse, parse_ollama_ndjson, parse_openai_sse};

#[cfg(test)]
mod http_tests {
    use super::*;
    use async_trait::async_trait;
    use jereko_core::{Message, MessageRole};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct RecordingHttp {
        inner: ReqwestHttpClient,
        calls: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl HttpClient for RecordingHttp {
        async fn request_json(
            &self,
            method: &str,
            url: &str,
            headers: &[(&str, String)],
            body: Option<serde_json::Value>,
        ) -> ProviderResult<serde_json::Value> {
            self.calls.lock().unwrap().push(format!("{method} {url}"));
            self.inner.request_json(method, url, headers, body).await
        }

        async fn request_text(
            &self,
            method: &str,
            url: &str,
            headers: &[(&str, String)],
            body: Option<serde_json::Value>,
        ) -> ProviderResult<String> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{method} TEXT {url}"));
            self.inner.request_text(method, url, headers, body).await
        }
    }

    #[tokio::test]
    async fn openai_complete_against_wiremock() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {"role": "assistant", "content": "hello from openai"},
                    "finish_reason": "stop"
                }]
            })))
            .mount(&server)
            .await;

        // SAFETY: single-threaded test; env mutation is scoped to this test.
        unsafe {
            std::env::set_var("OPENAI_API_KEY_COMPLETE_TEST", "test-key");
        }
        let http = Arc::new(ReqwestHttpClient::new());
        let provider = OpenAiProvider::new(http)
            .with_base_url(format!("{}/v1", server.uri()))
            .with_api_key_env("OPENAI_API_KEY_COMPLETE_TEST");
        let response = provider
            .complete(CompletionRequest {
                model: "gpt-4o-mini".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: Some(32),
            })
            .await
            .unwrap();
        assert_eq!(response.content, "hello from openai");
        unsafe {
            std::env::remove_var("OPENAI_API_KEY_COMPLETE_TEST");
        }
    }

    #[tokio::test]
    async fn openai_complete_stream_against_wiremock() {
        let server = MockServer::start().await;
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"hel\"},\"finish_reason\":null}]}\n\n\
data: {\"choices\":[{\"delta\":{\"content\":\"lo\"},\"finish_reason\":null}]}\n\n\
data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n\
data: [DONE]\n\n";
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse),
            )
            .mount(&server)
            .await;

        unsafe {
            std::env::set_var("OPENAI_API_KEY_STREAM_TEST", "test-key");
        }
        let http = Arc::new(ReqwestHttpClient::new());
        let provider = OpenAiProvider::new(http)
            .with_base_url(format!("{}/v1", server.uri()))
            .with_api_key_env("OPENAI_API_KEY_STREAM_TEST");
        let chunks = provider
            .complete_stream(CompletionRequest {
                model: "gpt-4o-mini".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: None,
            })
            .await
            .unwrap();
        let text: String = chunks.iter().map(|c| c.delta.as_str()).collect();
        assert_eq!(text, "hello");
        assert_eq!(
            chunks.last().unwrap().finish_reason.as_deref(),
            Some("stop")
        );
        unsafe {
            std::env::remove_var("OPENAI_API_KEY_STREAM_TEST");
        }
    }

    #[tokio::test]
    async fn anthropic_complete_against_wiremock() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [{"type": "text", "text": "hello from anthropic"}],
                "stop_reason": "end_turn"
            })))
            .mount(&server)
            .await;

        // SAFETY: single-threaded test; env mutation is scoped to this test.
        unsafe {
            std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        }
        let http = Arc::new(ReqwestHttpClient::new());
        let provider = AnthropicProvider::new(http).with_base_url(format!("{}/v1", server.uri()));
        let response = provider
            .complete(CompletionRequest {
                model: "claude-3-5-haiku-latest".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: None,
            })
            .await
            .unwrap();
        assert_eq!(response.content, "hello from anthropic");
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
    }

    #[tokio::test]
    async fn ollama_complete_against_wiremock() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {"role": "assistant", "content": "hello from ollama"}
            })))
            .mount(&server)
            .await;

        let http = Arc::new(ReqwestHttpClient::new());
        let provider = OllamaProvider::new(http).with_base_url(server.uri());
        let response = provider
            .complete(CompletionRequest {
                model: "llama3.2".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: None,
            })
            .await
            .unwrap();
        assert_eq!(response.content, "hello from ollama");
    }

    #[tokio::test]
    async fn ollama_complete_stream_against_wiremock() {
        let server = MockServer::start().await;
        let ndjson = "{\"message\":{\"content\":\"hi\"},\"done\":false}\n{\"message\":{\"content\":\"!\"},\"done\":true}\n";
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ndjson))
            .mount(&server)
            .await;

        let http = Arc::new(ReqwestHttpClient::new());
        let provider = OllamaProvider::new(http).with_base_url(server.uri());
        let chunks = provider
            .complete_stream(CompletionRequest {
                model: "llama3.2".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: None,
            })
            .await
            .unwrap();
        let text: String = chunks.iter().map(|c| c.delta.as_str()).collect();
        assert_eq!(text, "hi!");
    }

    #[tokio::test]
    async fn recording_client_tracks_calls() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": [{"name": "llama3.2"}]
            })))
            .mount(&server)
            .await;

        let http = Arc::new(RecordingHttp {
            inner: ReqwestHttpClient::new(),
            calls: Mutex::new(Vec::new()),
        });
        let provider = OllamaProvider::new(http.clone()).with_base_url(server.uri());
        let models = provider.list_models().await.unwrap();
        assert_eq!(models[0].id, "llama3.2");
        assert!(!http.calls.lock().unwrap().is_empty());
        let _map: HashMap<(), ()> = HashMap::new();
    }

    #[tokio::test]
    async fn stub_complete_stream_single_chunk() {
        let provider = StubProvider::new("openai");
        let chunks = provider
            .complete_stream(CompletionRequest {
                model: "stub-model".into(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "hi".into(),
                    provider: None,
                }],
                max_tokens: None,
            })
            .await
            .unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].delta.contains("stub:openai"));
    }
}
