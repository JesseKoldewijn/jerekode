//! Layer 2 in-process router integration tests.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use jereko_server::{build_router, AppState};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tower::ServiceExt;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/fixtures")
        .join(path)
}

fn load_json(path: &str) -> Value {
    let raw = fs::read_to_string(fixture(path)).unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[tokio::test]
async fn v1_create_session_via_router() {
    let app = build_router(AppState::default());
    let req_body = load_json("v1/create_session_request.json");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "active");
    assert_eq!(json["provider"], "anthropic");
    assert!(json["id"].is_string());
}

#[tokio::test]
async fn v2_session_lifecycle_via_router() {
    let app = build_router(AppState::default());

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    load_json("v2/create_session_request.json").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = create.into_body().collect().await.unwrap().to_bytes();
    let session: Value = serde_json::from_slice(&create_body).unwrap();
    let session_id = session["session"]["id"].as_str().unwrap();

    let message = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v2/sessions/{session_id}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    load_json("v2/send_message_request.json").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(message.status(), StatusCode::OK);
    let msg_body = message.into_body().collect().await.unwrap().to_bytes();
    let msg_json: Value = serde_json::from_slice(&msg_body).unwrap();
    assert_eq!(msg_json["content"], "[stub:anthropic] received 1 messages");
    assert_eq!(msg_json["session"]["messages"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_providers_returns_stubs() {
    let app = build_router(AppState::default());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v2/providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let ids: Vec<_> = json["providers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"openai"));
    assert!(ids.contains(&"anthropic"));
}
