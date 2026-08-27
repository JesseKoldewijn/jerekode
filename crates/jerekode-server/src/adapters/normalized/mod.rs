//! Canonical internal API types — the single shape handlers operate on.
//!
//! Both v1 and v2 adapters convert inbound requests into these types and
//! convert outbound responses back to their version-specific wire format.

use jerekode_core::{Message, Session, SessionId, SessionStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: SessionId,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub session: Session,
    pub assistant_message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProvidersResponse {
    pub providers: Vec<ProviderSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub id: String,
    pub display_name: String,
}

/// Map a core session to a normalized response wrapper.
pub fn session_response(session: Session) -> GetSessionResponse {
    GetSessionResponse { session }
}

/// Default status for newly created sessions.
pub fn new_session(provider_id: Option<String>) -> Session {
    let mut session = Session::new();
    session.provider_id = provider_id;
    session.status = SessionStatus::Active;
    session
}
