//! Layer 1 adapter unit tests — normalize/denormalize round-trips against fixtures.

use jereko_core::{Session, SessionId, SessionStatus};
use jereko_server::adapters::normalized;
use jereko_server::adapters::v1::{
    denormalize_create_session, denormalize_send_message, normalize_create_session,
    V1CreateSessionRequest,
};
use jereko_server::adapters::v2::{
    denormalize_create_session as v2_denorm_create, normalize_create_session as v2_norm_create,
    V2CreateSessionRequest,
};
use std::fs;
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/fixtures")
        .join(path)
}

#[test]
fn v1_create_session_request_matches_fixture_shape() {
    let raw = fs::read_to_string(fixture("v1/create_session_request.json")).unwrap();
    let req: V1CreateSessionRequest = serde_json::from_str(&raw).unwrap();
    let normalized = normalize_create_session(req);
    assert_eq!(normalized.provider_id.as_deref(), Some("anthropic"));
    assert_eq!(
        normalized.model.as_deref(),
        Some("claude-3-5-sonnet-20241022")
    );
}

#[test]
fn v1_create_session_response_matches_fixture_fields() {
    let session = Session {
        id: SessionId("00000000-0000-4000-8000-000000000001".into()),
        status: SessionStatus::Active,
        messages: vec![],
        provider_id: Some("anthropic".into()),
    };
    let resp = denormalize_create_session(normalized::CreateSessionResponse { session });
    assert_eq!(resp.status, "active");
    assert_eq!(resp.provider.as_deref(), Some("anthropic"));
}

#[test]
fn v1_send_message_response_content_from_fixture() {
    let session = Session {
        id: SessionId("00000000-0000-4000-8000-000000000001".into()),
        status: SessionStatus::Active,
        messages: vec![],
        provider_id: Some("anthropic".into()),
    };
    let assistant = jereko_core::Message {
        role: jereko_core::MessageRole::Assistant,
        content: "[stub:anthropic] received 1 messages".into(),
        provider: Some("anthropic".into()),
    };
    let resp = denormalize_send_message(normalized::SendMessageResponse {
        session,
        assistant_message: assistant,
    });
    assert_eq!(resp.content, "[stub:anthropic] received 1 messages");
}

#[test]
fn v2_create_session_request_matches_fixture_shape() {
    let raw = fs::read_to_string(fixture("v2/create_session_request.json")).unwrap();
    let req: V2CreateSessionRequest = serde_json::from_str(&raw).unwrap();
    let normalized = v2_norm_create(req);
    assert_eq!(normalized.provider_id.as_deref(), Some("anthropic"));
}

#[test]
fn v2_create_session_response_preserves_session() {
    let session = Session {
        id: SessionId("00000000-0000-4000-8000-000000000001".into()),
        status: SessionStatus::Active,
        messages: vec![],
        provider_id: Some("anthropic".into()),
    };
    let resp = v2_denorm_create(normalized::CreateSessionResponse {
        session: session.clone(),
    });
    assert_eq!(resp.session.id, session.id);
    assert_eq!(resp.session.status, SessionStatus::Active);
}
