use crate::adapters::normalized;
use crate::session_store::SessionStorePort;
use jereko_core::{Message, MessageRole};
use jereko_providers::{resolve, CompletionRequest, ProviderRegistry};
use std::sync::Arc;

pub struct HandlerContext {
    pub sessions: Arc<dyn SessionStorePort>,
    pub providers: Arc<ProviderRegistry>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
}

impl HandlerContext {
    pub fn create_session(
        &self,
        req: normalized::CreateSessionRequest,
    ) -> Result<normalized::CreateSessionResponse, HandlerError> {
        let provider_id = req
            .provider_id
            .or_else(|| self.default_provider.clone())
            .unwrap_or_else(|| "openai".into());

        if self.providers.get(&provider_id).is_none() {
            return Err(HandlerError::ProviderNotFound(provider_id));
        }

        let mut session = normalized::new_session(Some(provider_id));
        if req.model.is_some() {
            // Model stored implicitly via first completion; session has no model field yet.
        }
        let id = self.sessions.insert(session.clone());
        session.id = id;
        Ok(normalized::CreateSessionResponse { session })
    }

    pub fn get_session(
        &self,
        session_id: &str,
    ) -> Result<normalized::GetSessionResponse, HandlerError> {
        let id = jereko_core::SessionId(session_id.to_string());
        let session = self
            .sessions
            .get(&id)
            .ok_or_else(|| HandlerError::SessionNotFound(session_id.into()))?;
        Ok(normalized::GetSessionResponse { session })
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        content: String,
    ) -> Result<normalized::SendMessageResponse, HandlerError> {
        let id = jereko_core::SessionId(session_id.to_string());
        let mut session = self
            .sessions
            .get(&id)
            .ok_or_else(|| HandlerError::SessionNotFound(session_id.into()))?;

        let provider_id = session
            .provider_id
            .clone()
            .or_else(|| self.default_provider.clone())
            .unwrap_or_else(|| "openai".into());

        let provider = resolve(&self.providers, &provider_id)
            .map_err(|_| HandlerError::ProviderNotFound(provider_id.clone()))?;

        let user_message = Message {
            role: MessageRole::User,
            content: content.clone(),
            provider: None,
        };
        session.messages.push(user_message.clone());

        let model = self
            .default_model
            .clone()
            .unwrap_or_else(|| "stub-model".into());

        let completion = provider
            .complete(CompletionRequest {
                model,
                messages: session.messages.clone(),
                max_tokens: None,
            })
            .await
            .map_err(|e| HandlerError::Provider(e.to_string()))?;

        let assistant_message = Message {
            role: MessageRole::Assistant,
            content: completion.content.clone(),
            provider: Some(provider_id),
        };
        session.messages.push(assistant_message.clone());
        self.sessions.update(session.clone());

        Ok(normalized::SendMessageResponse {
            session,
            assistant_message,
        })
    }

    pub fn list_providers(&self) -> normalized::ListProvidersResponse {
        let providers = self
            .providers
            .ids()
            .map(|id| normalized::ProviderSummary {
                id: id.0.clone(),
                display_name: self
                    .providers
                    .get(&id.0)
                    .map(|p| p.display_name().to_string())
                    .unwrap_or_else(|| id.0.clone()),
            })
            .collect();
        normalized::ListProvidersResponse { providers }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    #[error("provider error: {0}")]
    Provider(String),
}
