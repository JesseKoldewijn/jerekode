//! Durable non-interactive agent loop for CLI `run` and future attach paths.
//!
//! Creates or reuses a session, calls the provider, executes tools when the
//! model returns tool calls, and loops until final text or `max_turns`.

use crate::session_store::SessionStorePort;
use crate::tools::{ToolCall, ToolExecutor, ToolName};
use jerekode_core::{Message, MessageRole, Session, SessionId};
use jerekode_providers::{CompletionRequest, ProviderRegistry, ProviderToolCall, resolve};
use std::sync::Arc;
use thiserror::Error;

const DEFAULT_MAX_TURNS: usize = 8;

#[derive(Debug, Error)]
pub enum AgentLoopError {
    #[error("provider not found: {0}")]
    ProviderNotFound(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("max turns exceeded ({0})")]
    MaxTurnsExceeded(usize),
    #[error("empty user message")]
    EmptyMessage,
}

pub type AgentLoopResult<T> = Result<T, AgentLoopError>;

#[derive(Debug, Clone)]
pub struct AgentRunRequest {
    pub message: String,
    pub session_id: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub max_turns: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AgentRunResult {
    pub session_id: String,
    pub final_text: String,
    pub turns: usize,
}

/// Coordinates session + provider + tools for one-shot / multi-turn agent runs.
pub struct AgentLoop {
    sessions: Arc<dyn SessionStorePort>,
    providers: Arc<ProviderRegistry>,
    tools: ToolExecutor,
    default_provider: Option<String>,
    default_model: Option<String>,
}

impl AgentLoop {
    pub fn new(
        sessions: Arc<dyn SessionStorePort>,
        providers: Arc<ProviderRegistry>,
        tools: ToolExecutor,
        default_provider: Option<String>,
        default_model: Option<String>,
    ) -> Self {
        Self {
            sessions,
            providers,
            tools,
            default_provider,
            default_model,
        }
    }

    pub async fn run(&self, req: AgentRunRequest) -> AgentLoopResult<AgentRunResult> {
        let message = req.message.trim();
        if message.is_empty() {
            return Err(AgentLoopError::EmptyMessage);
        }

        let max_turns = req.max_turns.unwrap_or(DEFAULT_MAX_TURNS).max(1);
        let provider_id = req
            .provider_id
            .clone()
            .or_else(|| self.default_provider.clone())
            .unwrap_or_else(|| "openai".into());
        let model = req
            .model
            .clone()
            .or_else(|| self.default_model.clone())
            .unwrap_or_else(|| "stub-model".into());

        let mut session = self.resolve_session(req.session_id.as_deref(), &provider_id)?;
        session.messages.push(Message {
            role: MessageRole::User,
            content: message.to_string(),
            provider: None,
        });

        let provider = resolve(&self.providers, &provider_id)
            .map_err(|_| AgentLoopError::ProviderNotFound(provider_id.clone()))?;

        let mut turns = 0usize;
        loop {
            turns += 1;
            if turns > max_turns {
                self.sessions.update(session);
                return Err(AgentLoopError::MaxTurnsExceeded(max_turns));
            }

            let response = provider
                .complete(CompletionRequest {
                    model: model.clone(),
                    messages: session.messages.clone(),
                    max_tokens: None,
                })
                .await
                .map_err(|e| AgentLoopError::Provider(e.to_string()))?;

            if response.tool_calls.is_empty() {
                let final_text = response.content;
                session.messages.push(Message {
                    role: MessageRole::Assistant,
                    content: final_text.clone(),
                    provider: Some(provider_id.clone()),
                });
                let session_id = session.id.0.clone();
                self.sessions.update(session);
                return Ok(AgentRunResult {
                    session_id,
                    final_text,
                    turns,
                });
            }

            // Record assistant turn that requested tools (content may be empty).
            if !response.content.is_empty() {
                session.messages.push(Message {
                    role: MessageRole::Assistant,
                    content: response.content,
                    provider: Some(provider_id.clone()),
                });
            } else {
                session.messages.push(Message {
                    role: MessageRole::Assistant,
                    content: format_tool_call_summary(&response.tool_calls),
                    provider: Some(provider_id.clone()),
                });
            }

            for call in &response.tool_calls {
                let tool_call = map_tool_call(call)?;
                let result = self.tools.execute(&tool_call);
                session.messages.push(Message {
                    role: MessageRole::Tool,
                    content: serde_json::json!({
                        "tool": call.name,
                        "ok": result.ok,
                        "output": result.output,
                    })
                    .to_string(),
                    provider: None,
                });
            }
        }
    }

    fn resolve_session(
        &self,
        session_id: Option<&str>,
        provider_id: &str,
    ) -> AgentLoopResult<Session> {
        if let Some(id) = session_id {
            let sid = SessionId(id.to_string());
            let mut session = self
                .sessions
                .get(&sid)
                .ok_or_else(|| AgentLoopError::SessionNotFound(id.to_string()))?;
            if session.provider_id.is_none() {
                session.provider_id = Some(provider_id.to_string());
            }
            return Ok(session);
        }

        let mut session = Session::new();
        session.provider_id = Some(provider_id.to_string());
        let id = self.sessions.insert(session.clone());
        session.id = id;
        Ok(session)
    }
}

fn format_tool_call_summary(calls: &[ProviderToolCall]) -> String {
    let names: Vec<&str> = calls.iter().map(|c| c.name.as_str()).collect();
    format!("tool_calls:{}", names.join(","))
}

fn map_tool_call(call: &ProviderToolCall) -> AgentLoopResult<ToolCall> {
    let name = match call.name.to_ascii_lowercase().as_str() {
        "read" => ToolName::Read,
        "write" => ToolName::Write,
        "edit" => ToolName::Edit,
        "bash" => ToolName::Bash,
        "grep" => ToolName::Grep,
        other => return Err(AgentLoopError::UnknownTool(other.to_string())),
    };
    Ok(ToolCall {
        name,
        args: call.arguments.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_store::SessionStore;
    use async_trait::async_trait;
    use jerekode_providers::{
        CompletionChunk, CompletionRequest, CompletionResponse, ModelInfo, Provider, ProviderId,
        ProviderResult,
    };
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    struct ScriptedProvider {
        id: ProviderId,
        turn: AtomicUsize,
    }

    impl ScriptedProvider {
        fn new() -> Self {
            Self {
                id: ProviderId::new("scripted"),
                turn: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl Provider for ScriptedProvider {
        fn id(&self) -> &ProviderId {
            &self.id
        }

        fn display_name(&self) -> &str {
            "Scripted"
        }

        async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
            Ok(vec![ModelInfo {
                id: "scripted-model".into(),
                display_name: Some("Scripted".into()),
                context_window: Some(8192),
            }])
        }

        async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
            let n = self.turn.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                Ok(CompletionResponse {
                    content: String::new(),
                    model: request.model,
                    finish_reason: Some("tool_calls".into()),
                    tool_calls: vec![ProviderToolCall {
                        id: Some("call_1".into()),
                        name: "read".into(),
                        arguments: serde_json::json!({ "path": "note.txt" }),
                    }],
                })
            } else {
                Ok(CompletionResponse {
                    content: "final answer from tools".into(),
                    model: request.model,
                    finish_reason: Some("stop".into()),
                    tool_calls: Vec::new(),
                })
            }
        }

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
    }

    fn loop_with(provider: Box<dyn Provider>, root: PathBuf) -> AgentLoop {
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        AgentLoop::new(
            Arc::new(SessionStore::new()),
            Arc::new(registry),
            ToolExecutor::new(root).with_bash(true),
            Some("scripted".into()),
            Some("scripted-model".into()),
        )
    }

    /// Clean up unused mut registry in stub test
    #[tokio::test]
    async fn agent_loop_returns_final_text_from_stub() {
        let dir = TempDir::new().unwrap();
        let agent = AgentLoop::new(
            Arc::new(SessionStore::new()),
            Arc::new(ProviderRegistry::with_stubs()),
            ToolExecutor::new(dir.path()).with_bash(true),
            Some("openai".into()),
            Some("stub-model".into()),
        );
        let result = agent
            .run(AgentRunRequest {
                message: "hello".into(),
                session_id: None,
                provider_id: Some("openai".into()),
                model: Some("stub-model".into()),
                max_turns: Some(3),
            })
            .await
            .unwrap();
        assert!(result.final_text.contains("stub:openai"));
        assert_eq!(result.turns, 1);
        assert!(!result.session_id.is_empty());
    }

    #[tokio::test]
    async fn agent_loop_executes_tools_then_finalizes() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.txt"), "payload").unwrap();
        let agent = loop_with(Box::new(ScriptedProvider::new()), dir.path().to_path_buf());
        let result = agent
            .run(AgentRunRequest {
                message: "read the note".into(),
                session_id: None,
                provider_id: Some("scripted".into()),
                model: Some("scripted-model".into()),
                max_turns: Some(4),
            })
            .await
            .unwrap();
        assert_eq!(result.final_text, "final answer from tools");
        assert_eq!(result.turns, 2);
    }

    #[tokio::test]
    async fn agent_loop_rejects_empty_message() {
        let dir = TempDir::new().unwrap();
        let agent = AgentLoop::new(
            Arc::new(SessionStore::new()),
            Arc::new(ProviderRegistry::with_stubs()),
            ToolExecutor::new(dir.path()),
            None,
            None,
        );
        let err = agent
            .run(AgentRunRequest {
                message: "   ".into(),
                session_id: None,
                provider_id: None,
                model: None,
                max_turns: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AgentLoopError::EmptyMessage));
    }

    #[tokio::test]
    async fn agent_loop_respects_max_turns() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.txt"), "x").unwrap();

        struct AlwaysTools {
            id: ProviderId,
        }

        #[async_trait]
        impl Provider for AlwaysTools {
            fn id(&self) -> &ProviderId {
                &self.id
            }
            fn display_name(&self) -> &str {
                "AlwaysTools"
            }
            async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
                Ok(vec![])
            }
            async fn complete(
                &self,
                request: CompletionRequest,
            ) -> ProviderResult<CompletionResponse> {
                Ok(CompletionResponse {
                    content: String::new(),
                    model: request.model,
                    finish_reason: Some("tool_calls".into()),
                    tool_calls: vec![ProviderToolCall {
                        id: None,
                        name: "read".into(),
                        arguments: serde_json::json!({ "path": "note.txt" }),
                    }],
                })
            }
        }

        let agent = loop_with(
            Box::new(AlwaysTools {
                id: ProviderId::new("scripted"),
            }),
            dir.path().to_path_buf(),
        );
        let err = agent
            .run(AgentRunRequest {
                message: "loop".into(),
                session_id: None,
                provider_id: Some("scripted".into()),
                model: Some("m".into()),
                max_turns: Some(2),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AgentLoopError::MaxTurnsExceeded(2)));
    }
}
