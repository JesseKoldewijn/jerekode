//! v2 HTTP adapter (stub).
//!
//! v2 is the forward-looking API surface. Normalization happens immediately
//! on ingress so core handlers remain version-agnostic.

mod routes;
mod types;

pub use routes::router;

pub use types::{
    V2CreateSessionRequest, V2CreateSessionResponse, V2ErrorResponse, V2SendMessageRequest,
    V2SendMessageResponse,
};

/// Convert v2 create-session request to normalized form.
pub fn normalize_create_session(
    req: V2CreateSessionRequest,
) -> crate::adapters::normalized::CreateSessionRequest {
    crate::adapters::normalized::CreateSessionRequest {
        provider_id: req.provider_id,
        model: req.model,
    }
}

/// Convert normalized create-session response to v2 wire format.
pub fn denormalize_create_session(
    resp: crate::adapters::normalized::CreateSessionResponse,
) -> V2CreateSessionResponse {
    V2CreateSessionResponse {
        session: resp.session,
    }
}

/// Convert normalized send-message response to v2 wire format.
pub fn denormalize_send_message(
    resp: crate::adapters::normalized::SendMessageResponse,
) -> V2SendMessageResponse {
    V2SendMessageResponse {
        session: resp.session,
        content: resp.assistant_message.content,
    }
}
