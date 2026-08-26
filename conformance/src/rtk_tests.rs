//! RTK dual-host rewrite conformance.
//!
//! First-party plugins must prove rewrite through **real** hosts (native dylib +
//! Bun process sidecar), not in-memory fakes. Table path only — no `rtk` binary.

use jereko_config::PluginEntry;
use jereko_plugins::{
    BunPluginHost, BunProcessSidecarPort, HookCall, NativePluginHost, PluginOrchestrator,
    SidecarOutbound, SidecarPort, TOOL_EXECUTE_BEFORE, apply_command_mutations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Arc;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn native_rtk_lib() -> PathBuf {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target")
        .join(profile);
    if cfg!(windows) {
        base.join("jereko_rtk_plugin.dll")
    } else if cfg!(target_os = "macos") {
        base.join("libjereko_rtk_plugin.dylib")
    } else {
        base.join("libjereko_rtk_plugin.so")
    }
}

fn sidecar_entry() -> PathBuf {
    repo_root().join("sidecar/src/index.ts")
}

fn rtk_bun_entry() -> PathBuf {
    repo_root().join("packages/rtk/src/index.ts")
}

fn bun_available() -> bool {
    StdCommand::new("bun")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn require_or_skip(path: &Path, label: &str) -> bool {
    if path.exists() {
        return true;
    }
    let msg = format!("{label} missing at {}", path.display());
    if std::env::var_os("CI").is_some() {
        panic!("{msg}");
    }
    eprintln!("skipping: {msg}");
    false
}

fn require_bun_or_skip() -> bool {
    if bun_available() {
        return true;
    }
    if std::env::var_os("CI").is_some() {
        panic!("bun required on PATH for first-party RTK Bun e2e");
    }
    eprintln!("skipping: bun not on PATH");
    false
}

#[tokio::test]
async fn rtk_native_rewrites_git_status_from_fixture() {
    let lib = native_rtk_lib();
    if !require_or_skip(&lib, "RTK native plugin") {
        return;
    }

    let fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture("plugins/rtk/rewrite_git_status.json")).unwrap(),
    )
    .unwrap();

    let native = Arc::new(NativePluginHost::with_library_path(lib.to_string_lossy()));
    let mut orch = PluginOrchestrator::new(vec![native]);
    orch.load_from_config(&[PluginEntry::Native {
        native: lib.to_string_lossy().into(),
    }])
    .await
    .unwrap();

    let results = orch
        .dispatch_hook(HookCall {
            hook: TOOL_EXECUTE_BEFORE.into(),
            payload: fixture["payload"].clone(),
        })
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    let command = apply_command_mutations(
        fixture["payload"]["command"].as_str().unwrap().into(),
        &results,
    );
    assert_eq!(command, fixture["expected_command"].as_str().unwrap());
    assert_eq!(results[0].output["host"], "native");
    assert_eq!(results[0].output["stub"], false);
}

#[tokio::test]
async fn rtk_bun_process_rewrites_git_status_from_fixture() {
    if !require_bun_or_skip() {
        return;
    }
    let entry = sidecar_entry();
    let rtk = rtk_bun_entry();
    if !require_or_skip(&entry, "sidecar entry") || !require_or_skip(&rtk, "RTK Bun entry") {
        return;
    }

    let fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture("plugins/rtk/rewrite_git_status.json")).unwrap(),
    )
    .unwrap();

    let process = BunProcessSidecarPort::spawn(entry.to_string_lossy().into_owned())
        .await
        .expect("spawn bun sidecar");
    process.wait_startup_ready().await.expect("startup ready");

    let bun = Arc::new(BunPluginHost::new(process.clone()));
    let mut orch = PluginOrchestrator::new(vec![bun]);
    let plugin_path = rtk.canonicalize().unwrap().to_string_lossy().into_owned();
    orch.load_from_config(&[PluginEntry::Bun(plugin_path)])
        .await
        .expect("load @jerekode/rtk via path");

    let results = orch
        .dispatch_hook(HookCall {
            hook: fixture["hook"].as_str().unwrap().into(),
            payload: fixture["payload"].clone(),
        })
        .await
        .expect("dispatch");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].output["host"], "bun");
    assert_eq!(results[0].output["stub"], false);
    let command = apply_command_mutations(
        fixture["payload"]["command"].as_str().unwrap().into(),
        &results,
    );
    assert_eq!(command, fixture["expected_command"].as_str().unwrap());

    let _ = process.send(SidecarOutbound::Shutdown).await;
}

#[tokio::test]
async fn rtk_bun_process_and_native_agree_on_git_status_rewrite() {
    if !require_bun_or_skip() {
        return;
    }
    let lib = native_rtk_lib();
    let entry = sidecar_entry();
    let rtk = rtk_bun_entry();
    if !require_or_skip(&lib, "RTK native plugin")
        || !require_or_skip(&entry, "sidecar entry")
        || !require_or_skip(&rtk, "RTK Bun entry")
    {
        return;
    }

    let fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture("plugins/rtk/rewrite_git_status.json")).unwrap(),
    )
    .unwrap();
    let expected = fixture["expected_command"].as_str().unwrap();
    let original = fixture["payload"]["command"].as_str().unwrap().to_string();

    let process = BunProcessSidecarPort::spawn(entry.to_string_lossy().into_owned())
        .await
        .expect("spawn bun sidecar");
    process.wait_startup_ready().await.expect("startup ready");

    let bun = Arc::new(BunPluginHost::new(process.clone()));
    let native = Arc::new(NativePluginHost::with_library_path(lib.to_string_lossy()));
    let mut orch = PluginOrchestrator::new(vec![native, bun]);
    let plugin_path = rtk.canonicalize().unwrap().to_string_lossy().into_owned();
    orch.load_from_config(&[
        PluginEntry::Native {
            native: lib.to_string_lossy().into(),
        },
        PluginEntry::Bun(plugin_path),
    ])
    .await
    .expect("load dual hosts");

    let results = orch
        .dispatch_hook(HookCall {
            hook: TOOL_EXECUTE_BEFORE.into(),
            payload: fixture["payload"].clone(),
        })
        .await
        .expect("dispatch");
    assert!(
        results.len() >= 2,
        "expected native + bun results, got {}",
        results.len()
    );
    assert!(results.iter().any(|r| r.output["host"] == "native"));
    assert!(results.iter().any(|r| r.output["host"] == "bun"));
    assert!(results.iter().all(|r| r.output["stub"] != true));

    let mutated = apply_command_mutations(original, &results);
    assert_eq!(mutated, expected);

    let _ = process.send(SidecarOutbound::Shutdown).await;
}

#[tokio::test]
async fn rtk_server_execute_tool_runs_with_orchestrator_attached() {
    let lib = native_rtk_lib();
    if !require_or_skip(&lib, "RTK native plugin") {
        return;
    }

    use jereko_config::OpenCodeConfig;
    use jereko_server::AppState;
    use jereko_server::tools::{ToolCall, ToolName};

    let native = Arc::new(NativePluginHost::with_library_path(lib.to_string_lossy()));
    let mut orch = PluginOrchestrator::new(vec![native]);
    orch.load_from_config(&[PluginEntry::Native {
        native: lib.to_string_lossy().into(),
    }])
    .await
    .unwrap();

    let state = AppState::new(&OpenCodeConfig::default()).with_plugins(orch);
    // Passthrough command — verifies hook path does not break execution.
    let result = state
        .ctx
        .execute_tool(ToolCall {
            name: ToolName::Bash,
            args: serde_json::json!({"command": "echo jereko-rtk"}),
        })
        .await;
    assert!(
        result.ok,
        "bash after hooks should succeed: {}",
        result.output
    );
    assert!(result.output.contains("jereko-rtk"));
}
