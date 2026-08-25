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
