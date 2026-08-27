//! Shared helpers for CLI integration tests.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../conformance/fixtures/cli")
}

pub fn load_json(name: &str) -> Value {
    let path = fixture_dir().join(name);
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture {path:?}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse fixture {path:?}: {e}"))
}

pub fn load_text(name: &str) -> String {
    let path = fixture_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture {path:?}: {e}"))
}

pub fn run_cli(argv: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jerekode"));
    cmd.args(argv);
    for (key, val) in env {
        cmd.env(key, val);
    }
    cmd.output().expect("spawn jerekode")
}

pub fn assert_exit(output: &Output, expect_success: bool) {
    if expect_success {
        assert!(
            output.status.success(),
            "expected success; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        assert!(
            !output.status.success(),
            "expected failure; stdout={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

pub fn assert_stdout_contains(stdout: &str, needles: &[Value]) {
    for needle in needles {
        let s = needle
            .as_str()
            .expect("stdout_contains entry must be string");
        assert!(stdout.contains(s), "stdout missing {s:?}: {stdout}");
    }
}

pub fn assert_stdout_contains_ignore_case(stdout: &str, needles: &[Value]) {
    let lower = stdout.to_lowercase();
    for needle in needles {
        let s = needle
            .as_str()
            .expect("stdout_contains_ignore_case entry must be string");
        assert!(
            lower.contains(&s.to_lowercase()),
            "stdout missing {s:?}: {stdout}"
        );
    }
}

pub fn assert_stdout_must_not_contain_line_prefix(stdout: &str, prefixes: &[Value]) {
    for prefix in prefixes {
        let p = prefix
            .as_str()
            .expect("stdout_must_not_contain_line_prefix entry must be string");
        for line in stdout.lines() {
            let trimmed = line.trim_start();
            assert!(
                !trimmed.to_lowercase().starts_with(&p.to_lowercase()),
                "help must omit unfinished {p:?} line: {line}"
            );
        }
    }
}

pub fn env_pairs_from_fixture(obj: &Value) -> Vec<(String, String)> {
    obj.get("env")
        .and_then(|v| v.as_object())
        .map(|map| {
            map.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn argv_from_fixture(obj: &Value) -> Vec<String> {
    obj.get("argv")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn run_fixture_case(name: &str) -> Output {
    let fixture = load_json(name);
    let argv = argv_from_fixture(&fixture);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jerekode"));
    cmd.args(&argv);
    for (key, val) in env_pairs_from_fixture(&fixture) {
        cmd.env(key, val);
    }
    cmd.output().expect("spawn jerekode")
}

pub fn assert_fixture_run(name: &str) {
    let fixture = load_json(name);
    let output = run_fixture_case(name);
    let expect_success = fixture
        .get("exit_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    assert_exit(&output, expect_success);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(needles) = fixture.get("stdout_contains").and_then(|v| v.as_array()) {
        assert_stdout_contains(&stdout, needles);
    }
    if let Some(needles) = fixture
        .get("stdout_lines_contain")
        .and_then(|v| v.as_array())
    {
        assert_stdout_contains(&stdout, needles);
    }
}

pub fn spawn_serve(
    host_flag: &str,
    host: &str,
    port: u16,
    extra_env: &[(&str, &str)],
) -> std::process::Child {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jerekode"));
    cmd.args(["serve", host_flag, host, "--port", &port.to_string()])
        .env("JEREKO_USE_STUB_PROVIDERS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (key, val) in extra_env {
        cmd.env(key, val);
    }
    cmd.spawn().expect("spawn jerekode serve")
}

pub fn assert_session_list_shape(body: &Value, fixture_name: &str) {
    let shape = load_json(fixture_name);
    for key in shape
        .get("required_top_level_keys")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let k = key.as_str().unwrap();
        assert!(body.get(k).is_some(), "missing key {k} in {body}");
    }
    let min = shape
        .get("sessions_min_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let count = body
        .get("sessions")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert!(
        count >= min as usize,
        "expected >= {min} sessions in {body}"
    );
}
