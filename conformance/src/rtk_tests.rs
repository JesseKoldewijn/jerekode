//! RTK dual-host rewrite conformance (table path — no `rtk` binary required).

use jereko_config::PluginEntry;
use jereko_plugins::{
    BunPluginHost, HookCall, InMemorySidecarPort, NativePluginHost, PluginOrchestrator,
    TOOL_EXECUTE_BEFORE, apply_command_mutations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
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

fn require_native_lib(path: &Path) {
    if path.exists() {
        return;
    }
    if std::env::var_os("CI").is_some() {
        panic!(
            "RTK native plugin missing at {} — run: cargo build -p jereko-rtk-plugin --locked",
            path.display()
        );
    }
    eprintln!("skipping RTK native test: missing {}", path.display());
}

#[tokio::test]
async fn rtk_native_rewrites_git_status_from_fixture() {
    let lib = native_rtk_lib();
    if !lib.exists() {
        require_native_lib(&lib);
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
}

#[tokio::test]
async fn rtk_bun_inmemory_and_native_agree_on_passthrough() {
    let lib = native_rtk_lib();
    if !lib.exists() {
        require_native_lib(&lib);
        return;
    }

    let fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture("plugins/rtk/rewrite_passthrough.json")).unwrap(),
    )
    .unwrap();

    let port = Arc::new(InMemorySidecarPort::new());
    let bun = Arc::new(BunPluginHost::new(port));
    let native = Arc::new(NativePluginHost::with_library_path(lib.to_string_lossy()));
    let mut orch = PluginOrchestrator::new(vec![native, bun]);
    orch.load_from_config(&[
        PluginEntry::Native {
            native: lib.to_string_lossy().into(),
        },
        PluginEntry::Bun("@jerekode/rtk".into()),
    ])
    .await
    .unwrap();

    let results = orch
        .dispatch_hook(HookCall {
            hook: fixture["hook"].as_str().unwrap().into(),
            payload: fixture["payload"].clone(),
        })
        .await
        .unwrap();
    assert!(results.len() >= 2);
    let expected = fixture["expected_command"].as_str().unwrap();
    let original = fixture["payload"]["command"].as_str().unwrap().to_string();
    let mutated = apply_command_mutations(original, &results);
    assert_eq!(mutated, expected);
}

#[tokio::test]
async fn rtk_server_execute_tool_runs_with_orchestrator_attached() {
    let lib = native_rtk_lib();
    if !lib.exists() {
        require_native_lib(&lib);
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
