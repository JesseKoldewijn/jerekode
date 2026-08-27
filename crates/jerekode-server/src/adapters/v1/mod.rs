//! v1 HTTP adapter (stub).
//!
//! Converts v1 wire format ↔ `adapters::normalized` types at the boundary.
//! TODO(phase-1): implement full v1 route surface from owned conformance fixtures.

mod routes;
mod types;

pub use routes::router;

pub use types::{
    V1CreateSessionRequest, V1CreateSessionResponse, V1ErrorResponse, V1ListProvidersResponse,
    V1ProviderSummary, V1SendMessageRequest, V1SendMessageResponse,
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
        status: crate::adapters::status::session_status_str(resp.session.status).into(),
        provider: resp.session.provider_id,
    }
}

/// Convert v1 send-message request to normalized content string.
pub fn normalize_send_message(req: V1SendMessageRequest) -> String {
    req.content
}

/// Convert normalized send-message response to v1 wire format.
pub fn denormalize_send_message(
    resp: crate::adapters::normalized::SendMessageResponse,
) -> V1SendMessageResponse {
    V1SendMessageResponse {
        content: resp.assistant_message.content,
        session_id: resp.session.id.0,
    }
}

/// Convert normalized provider list to v1 wire format.
pub fn denormalize_list_providers(
    resp: crate::adapters::normalized::ListProvidersResponse,
) -> V1ListProvidersResponse {
    V1ListProvidersResponse {
        providers: resp
            .providers
            .into_iter()
            .map(|p| V1ProviderSummary {
                id: p.id,
                name: p.display_name,
            })
            .collect(),
    }
}
