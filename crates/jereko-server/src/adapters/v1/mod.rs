//! v1 HTTP adapter (stub).
//!
//! Converts v1 wire format ↔ `adapters::normalized` types at the boundary.
//! TODO(phase-1): implement full v1 route surface from owned conformance fixtures.

mod routes;
mod types;

pub use routes::router;

pub use types::{
    V1CreateSessionRequest, V1CreateSessionResponse, V1ErrorResponse, V1SendMessageRequest,
    V1SendMessageResponse,
};

/// Convert v1 create-session request to normalized form.
pub fn normalize_create_session(
    req: V1CreateSessionRequest,
) -> crate::adapters::normalized::CreateSessionRequest {
    crate::adapters::normalized::CreateSessionRequest {
        provider_id: req.provider,
        model: req.model,
    }
}

/// Convert normalized create-session response to v1 wire format.
pub fn denormalize_create_session(
    resp: crate::adapters::normalized::CreateSessionResponse,
) -> V1CreateSessionResponse {
    V1CreateSessionResponse {
        id: resp.session.id.0,
        status: format!("{:?}", resp.session.status).to_lowercase(),
        provider: resp.session.provider_id,
    }
}
