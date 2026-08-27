//! CLI runtime smoke — spawn the real `jerekode` binary (not in-process router).

use serde_json::Value;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn jerekode_bin() -> &'static str {
    env!("CARGO_BIN_EXE_jerekode")
}

fn pick_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_health(client: &reqwest::Client, port: u16) {
    let url = format!("http://127.0.0.1:{port}/health");
    for _ in 0..50 {
        if let Ok(resp) = client.get(&url).send().await
            && resp.status().is_success()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("jerekode serve did not become healthy on port {port}");
}

#[test]
fn cli_version_prints_package_version() {
    let output = Command::new(jerekode_bin())
        .arg("version")
        .output()
        .expect("spawn jerekode version");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout missing version: {stdout}"
    );
    assert!(stdout.contains("jerekode"), "stdout: {stdout}");
}

#[tokio::test]
async fn cli_serve_health_and_v1_v2_session_smoke() {
    let port = pick_port();
    let child = Command::new(jerekode_bin())
        .args(["serve", "--host", "127.0.0.1", "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn jerekode serve");
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    wait_health(&client, port).await;

    let health = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .unwrap();
    assert!(health.status().is_success());
    assert_eq!(health.text().await.unwrap(), "ok");

    let v1 = client
        .post(format!("http://127.0.0.1:{port}/v1/session"))
        .json(&serde_json::json!({
            "provider": "anthropic",
            "model": "claude-3-5-sonnet-20241022"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(v1.status(), 201, "v1 create session");
    let v1_body: Value = v1.json().await.unwrap();
    assert_eq!(v1_body["status"], "active");
    let v1_id = v1_body["id"].as_str().unwrap_or_default();
    assert!(!v1_id.is_empty(), "v1 session id missing: {v1_body}");

    let v2 = client
        .post(format!("http://127.0.0.1:{port}/v2/sessions"))
        .json(&serde_json::json!({
            "provider_id": "anthropic",
            "model": "claude-3-5-sonnet-20241022"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(v2.status(), 201, "v2 create session");
    let v2_body: Value = v2.json().await.unwrap();
    assert_eq!(v2_body["session"]["status"], "active");
    let v2_id = v2_body["session"]["id"].as_str().unwrap_or_default();
    assert!(!v2_id.is_empty(), "v2 session id missing: {v2_body}");
}

#[test]
fn cli_help_exits_zero() {
    let output = Command::new(jerekode_bin())
        .arg("--help")
        .output()
        .expect("spawn jerekode --help");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.to_lowercase().contains("serve"), "stdout: {stdout}");
    assert!(
        stdout.to_lowercase().contains("version"),
        "stdout: {stdout}"
    );
}
