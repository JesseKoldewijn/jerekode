//! Layer 3 black-box HTTP conformance against a running server.

use jerekode_config::OpenCodeConfig;
use jerekode_server::serve_on;
use serde_json::Value;
use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::time::Duration;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
}

fn load_json(path: &str) -> Value {
    let raw = fs::read_to_string(fixture(path)).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn pick_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::test]
async fn blackbox_v1_health_and_session() {
    let port = pick_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let config = OpenCodeConfig::default();

    tokio::spawn(async move {
        serve_on(addr, &config).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let health = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .unwrap();
    assert!(health.status().is_success());
    assert_eq!(health.text().await.unwrap(), "ok");

    let create = client
        .post(format!("http://127.0.0.1:{port}/v1/session"))
        .json(&load_json("v1/create_session_request.json"))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 201);
    let body: Value = create.json().await.unwrap();
    assert_eq!(body["status"], "active");
    assert_eq!(body["provider"], "anthropic");
}
