//! Layer 4 end-to-end session + plugin orchestrator flow.

use jereko_config::{ConfigLoader, PluginEntry};
use jereko_plugins::{
    BunPluginHost, HookCall, InMemorySidecarPort, NativePluginHost, PluginOrchestrator,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
}

#[tokio::test]
async fn e2e_config_session_plugin_flow() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".opencode");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("opencode.json"),
        r#"{"provider":"anthropic","plugins":["@acme/server-plugin",{"native":"./tools.so"}]}"#,
    )
    .unwrap();

    let mut loader = ConfigLoader::new();
    loader
        .load_file(
            config_dir.join("opencode.json"),
            jereko_config::ConfigLayer::Project,
        )
        .unwrap();
    assert_eq!(loader.opencode().provider.as_deref(), Some("anthropic"));

    let port = Arc::new(InMemorySidecarPort::new());
    let bun = Arc::new(BunPluginHost::new(port));
    let native = Arc::new(NativePluginHost::new("./tools.so"));
    let mut orchestrator = PluginOrchestrator::new(vec![native, bun]);
    orchestrator
        .load_from_config(loader.opencode().plugins.as_slice())
        .await
        .unwrap();

    let hook_fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture("plugins/hook_before_transform.json")).unwrap(),
    )
    .unwrap();

    let results = orchestrator
        .dispatch_hook(HookCall {
            hook: hook_fixture["hook"].as_str().unwrap().into(),
            payload: hook_fixture["payload"].clone(),
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.plugin.contains("acme")));
}

#[test]
fn plugin_entry_parsing_from_config() {
    let entries = [
        PluginEntry::Bun("@acme/server-plugin".into()),
        PluginEntry::Native {
            native: "./tools.so".into(),
        },
    ];
    assert_eq!(entries[0].display_name(), "@acme/server-plugin");
    assert_eq!(entries[1].display_name(), "./tools.so");
}
