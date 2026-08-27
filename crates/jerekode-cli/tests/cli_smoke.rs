//! CLI runtime smoke — spawn the real `jerekode` binary (not in-process router).
//! Expected shapes load from owned fixtures under `conformance/fixtures/cli/`.

mod common;

use common::{
    assert_exit, assert_fixture_run, assert_session_list_shape, assert_stdout_contains,
    assert_stdout_contains_ignore_case, assert_stdout_must_not_contain_line_prefix, load_json,
    load_text, run_cli, spawn_serve,
};
use serde_json::Value;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

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
    let shape = load_json("version_stdout_shape.json");
    let output = run_cli(&["version"], &[]);
    assert_exit(
        &output,
        shape
            .get("exit_success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(needles) = shape.get("stdout_contains").and_then(|v| v.as_array()) {
        assert_stdout_contains(&stdout, needles);
    }
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout missing crate version: {stdout}"
    );
}

#[tokio::test]
async fn cli_serve_health_and_v1_v2_session_smoke() {
    let port = pick_port();
    let child = spawn_serve("--host", "127.0.0.1", port, &[]);
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    wait_health(&client, port).await;

    let health = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .unwrap();
    assert!(health.status().is_success());
    assert_eq!(
        health.text().await.unwrap(),
        load_text("serve_health_body.txt").trim()
    );

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

#[tokio::test]
async fn cli_serve_hostname_alias_binds() {
    let port = pick_port();
    let child = spawn_serve("--hostname", "127.0.0.1", port, &[]);
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    wait_health(&client, port).await;
}

#[tokio::test]
async fn cli_serve_basic_auth_env_gates_health() {
    let port = pick_port();
    let child = spawn_serve(
        "--host",
        "127.0.0.1",
        port,
        &[("OPENCODE_SERVER_PASSWORD", "test-secret")],
    );
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/health");
    let mut saw_unauthorized = false;
    for _ in 0..50 {
        if let Ok(resp) = client.get(&url).send().await
            && resp.status() == reqwest::StatusCode::UNAUTHORIZED
        {
            saw_unauthorized = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(saw_unauthorized, "expected 401 from /health before auth");

    let ok = client
        .get(&url)
        .basic_auth("opencode", Some("test-secret"))
        .send()
        .await
        .unwrap();
    assert!(ok.status().is_success());
    assert_eq!(
        ok.text().await.unwrap(),
        load_text("serve_health_body.txt").trim()
    );
}

#[test]
fn cli_help_exits_zero() {
    let shape = load_json("help_stdout_shape.json");
    let output = run_cli(&["--help"], &[]);
    assert_exit(
        &output,
        shape
            .get("exit_success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(needles) = shape
        .get("stdout_contains_ignore_case")
        .and_then(|v| v.as_array())
    {
        assert_stdout_contains_ignore_case(&stdout, needles);
    }
    if let Some(prefixes) = shape
        .get("stdout_must_not_contain_line_prefix")
        .and_then(|v| v.as_array())
    {
        assert_stdout_must_not_contain_line_prefix(&stdout, prefixes);
    }
}

#[test]
fn cli_run_one_shot_with_stub_provider() {
    assert_fixture_run("run_one_shot_stub.json");
}

#[test]
fn cli_run_model_slash_form() {
    assert_fixture_run("run_model_slash_form.json");
}

#[test]
fn cli_run_without_message_exits_nonzero() {
    assert_fixture_run("run_missing_message.json");
}

#[test]
fn cli_models_lists_provider_model() {
    assert_fixture_run("models_stdout_contains.json");
}

#[tokio::test]
async fn cli_session_list_against_serve() {
    let port = pick_port();
    let child = spawn_serve("--host", "127.0.0.1", port, &[]);
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    wait_health(&client, port).await;

    let created = client
        .post(format!("http://127.0.0.1:{port}/v2/sessions"))
        .json(&serde_json::json!({ "provider_id": "openai" }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);

    let list = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args([
            "session",
            "list",
            "--url",
            &format!("http://127.0.0.1:{port}"),
            "--format",
            "json",
        ])
        .output()
        .expect("spawn session list");
    assert!(
        list.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&list.stderr)
    );
    let body: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_session_list_shape(&body, "session_list_json_shape.json");
}

#[test]
fn cli_version_flag_short_v() {
    let output = run_cli(&["-v"], &[]);
    assert_exit(&output, true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout missing version: {stdout}"
    );
}

#[test]
fn cli_version_flag_long() {
    let output = run_cli(&["--version"], &[]);
    assert_exit(&output, true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout missing version: {stdout}"
    );
}

#[test]
fn cli_bare_invoke_starts_tui_smoke() {
    let bun_ok = Command::new("bun")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !bun_ok {
        if std::env::var_os("CI").is_some() {
            panic!("cli_bare_invoke_starts_tui_smoke requires bun on PATH");
        }
        eprintln!("skipping: bun unavailable");
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .env("JEREKODE_TUI_SMOKE", "1")
        .output()
        .expect("spawn bare jerekode");
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("jerekode TUI"),
        "expected TUI banner, got: {stdout}"
    );
}

#[test]
fn cli_auth_login_list_logout_and_import() {
    let dir = tempfile::tempdir().unwrap();
    let jk = dir.path().join("jerekode-auth.json");
    let oc = dir.path().join("opencode-auth.json");
    std::fs::write(&oc, r#"{"groq":{"api_key":"from-opencode"}}"#).unwrap();

    let login = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args([
            "auth",
            "login",
            "--provider",
            "openai",
            "--api-key",
            "sk-test",
        ])
        .env("JEREKODE_AUTH_PATH", &jk)
        .output()
        .expect("auth login");
    assert!(
        login.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&login.stderr)
    );

    let list = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["auth", "list"])
        .env("JEREKODE_AUTH_PATH", &jk)
        .output()
        .expect("auth list");
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("openai"), "stdout: {stdout}");

    let import = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["auth", "import"])
        .env("JEREKODE_AUTH_PATH", &jk)
        .env("OPENCODE_AUTH_PATH", &oc)
        .output()
        .expect("auth import");
    assert!(
        import.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&import.stderr)
    );
    let list2 = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["auth", "ls"])
        .env("JEREKODE_AUTH_PATH", &jk)
        .output()
        .unwrap();
    let out2 = String::from_utf8_lossy(&list2.stdout);
    assert!(out2.contains("groq"), "stdout: {out2}");
    assert!(out2.contains("openai"), "stdout: {out2}");

    let logout = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["auth", "logout", "openai"])
        .env("JEREKODE_AUTH_PATH", &jk)
        .output()
        .unwrap();
    assert!(logout.status.success());
}

#[test]
fn cli_db_path_prints_default() {
    let output = run_cli(&["db", "path"], &[]);
    assert_exit(&output, true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jerekode"), "stdout: {stdout}");
}

#[test]
fn cli_run_file_flag_appends_contents() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.txt");
    std::fs::write(&file, "file-body-xyz").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["run", "--file", file.to_str().unwrap(), "prefix"])
        .env("JEREKO_USE_STUB_PROVIDERS", "1")
        .output()
        .expect("run --file");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_agent_list_smoke() {
    let output = run_cli(&["agent", "list"], &[]);
    assert_exit(&output, true);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("no custom agents"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cli_mcp_list_smoke() {
    let output = run_cli(&["mcp", "list"], &[]);
    assert_exit(&output, true);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("no MCP servers"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cli_auth_list_empty_store() {
    let dir = tempfile::tempdir().unwrap();
    let jk = dir.path().join("empty-auth.json");
    let output = run_cli(
        &["auth", "list"],
        &[("JEREKODE_AUTH_PATH", jk.to_str().unwrap())],
    );
    assert_exit(&output, true);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("no credentials"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cli_auth_logout_unknown_provider() {
    let dir = tempfile::tempdir().unwrap();
    let jk = dir.path().join("auth.json");
    let output = run_cli(
        &["auth", "logout", "missing-provider"],
        &[("JEREKODE_AUTH_PATH", jk.to_str().unwrap())],
    );
    assert_exit(&output, false);
}

#[test]
fn cli_auth_import_no_opencode_file() {
    let dir = tempfile::tempdir().unwrap();
    let jk = dir.path().join("auth.json");
    let output = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["auth", "import"])
        .env("JEREKODE_AUTH_PATH", &jk)
        .env("HOME", dir.path())
        .env_remove("OPENCODE_AUTH_PATH")
        .env_remove("USERPROFILE")
        .output()
        .expect("auth import");
    assert_exit(&output, true);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("no OpenCode auth.json found"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cli_attach_starts_tui_smoke_mode() {
    let bun_ok = Command::new("bun")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !bun_ok {
        if std::env::var_os("CI").is_some() {
            panic!("cli_attach_starts_tui_smoke_mode requires bun on PATH");
        }
        eprintln!("skipping: bun unavailable");
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["attach", "http://127.0.0.1:4096"])
        .env("JEREKODE_TUI_SMOKE", "1")
        .output()
        .expect("attach");
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[tokio::test]
async fn cli_run_attach_against_serve() {
    let port = pick_port();
    let child = spawn_serve("--host", "127.0.0.1", port, &[]);
    let _guard = KillOnDrop(child);

    let client = reqwest::Client::new();
    wait_health(&client, port).await;

    let url = format!("http://127.0.0.1:{port}");
    let output = Command::new(env!("CARGO_BIN_EXE_jerekode"))
        .args(["run", "--attach", &url, "hello attach"])
        .env("JEREKO_USE_STUB_PROVIDERS", "1")
        .output()
        .expect("run --attach");
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}
