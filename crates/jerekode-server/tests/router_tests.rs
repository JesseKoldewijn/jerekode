//! Layer 2 in-process router integration tests.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use jerekode_server::{AppState, build_router};
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

#[tokio::test]
async fn tools_execute_write_read_via_router() {
    let dir = tempfile::TempDir::new().unwrap();
    // ToolExecutor uses process cwd; run relative writes inside temp by changing cwd.
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let app = build_router(AppState::default());
    let write_body = load_json("tools/execute_write_request.json");
    let write = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/tools/execute")
                .header("content-type", "application/json")
                .body(Body::from(write_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(write.status(), StatusCode::OK);

    let read_body = load_json("tools/execute_read_request.json");
    let read = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v2/tools/execute")
                .header("content-type", "application/json")
                .body(Body::from(read_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    let body = read.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["output"], "parity-tools");

    std::env::set_current_dir(prev).unwrap();
}
#[tokio::test]
async fn extensions_mcp_call_and_lsp_hover() {
    let app = build_router(AppState::default());
    let mcp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extensions/mcp/call")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"tool":"mcp_echo","args":{"ok":true}}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mcp.status(), StatusCode::OK);
    let body = mcp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], true);

    let init = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extensions/lsp/initialize")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"root_uri":"file:///tmp"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(init.status(), StatusCode::OK);

    let hover = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extensions/lsp/hover")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "uri":"file:///tmp/a.rs",
                        "line":0,
                        "character":3,
                        "text":"fn foo() {}"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hover.status(), StatusCode::OK);
}

#[tokio::test]
async fn session_list_get_delete_via_router() {
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
    let session_id = session["session"]["id"].as_str().unwrap().to_string();

    let send = app
        .clone()
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
    assert_eq!(send.status(), StatusCode::OK);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v2/sessions/{session_id}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed_body = listed.into_body().collect().await.unwrap().to_bytes();
    let listed_json: Value = serde_json::from_slice(&listed_body).unwrap();
    assert_eq!(listed_json["messages"].as_array().unwrap().len(), 2);
    let _shape = load_json("v2/list_messages_response_shape.json");

    let sessions = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v2/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sessions.status(), StatusCode::OK);
    let sessions_body = sessions.into_body().collect().await.unwrap().to_bytes();
    let sessions_json: Value = serde_json::from_slice(&sessions_body).unwrap();
    let ids = sessions_json["sessions"].as_array().unwrap();
    assert!(ids.iter().any(|v| v.as_str() == Some(session_id.as_str())));

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v2/sessions/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let gone = app
        .oneshot(
            Request::builder()
                .uri(format!("/v2/sessions/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}
