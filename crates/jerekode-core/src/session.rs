use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque session identifier shared across HTTP adapters and the sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Idle,
    Completed,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    /// Provider/model that produced this message, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Canonical in-memory session representation used by the core runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub status: SessionStatus,
    pub messages: Vec<Message>,
    /// Active provider id for this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            status: SessionStatus::Active,
            messages: Vec::new(),
            provider_id: None,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_round_trips_json() {
        let session = Session::new();
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, restored.id);
    }

    #[test]
    fn session_and_id_default_construct() {
        let id = SessionId::default();
        assert!(!id.0.is_empty());
        let session = Session::default();
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.messages.is_empty());
        assert!(!session.id.0.is_empty());
    }
}
