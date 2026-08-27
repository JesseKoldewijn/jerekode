use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};
#[cfg(feature = "bun-sidecar")]
use jerekode_plugins::{BunPluginHost, BunProcessSidecarPort, SidecarOutbound, SidecarPort};
use jerekode_plugins::{NativePluginHost, PluginOrchestrator, WasmPluginHost};
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Provider id override
    #[arg(long)]
    pub provider: Option<String>,

    /// Model id override
    #[arg(long)]
    pub model: Option<String>,

    /// Project root for config discovery
    #[arg(long)]
    pub project: Option<String>,
}

pub async fn execute(args: RunArgs) -> anyhow::Result<()> {
    let project = args
        .project
        .map(Into::into)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));

    let cli = CliOverrides {
        provider: args.provider,
        model: args.model,
        ..Default::default()
    };
    let loader = ConfigLoader::load_discovered(&project, &cli)?;

    #[cfg(feature = "bun-sidecar")]
    {
        let sidecar_entry = loader
            .tui()
            .sidecar
            .as_ref()
            .and_then(|s| s.entry.clone())
            .unwrap_or_else(|| "sidecar/src/index.ts".into());

        let process = BunProcessSidecarPort::spawn(sidecar_entry).await?;
        process.wait_startup_ready().await?;
        let port: Arc<dyn SidecarPort> = process;
        let bun = Arc::new(BunPluginHost::new(port.clone()));
        let native = Arc::new(NativePluginHost::new());
        let wasm = Arc::new(WasmPluginHost::new());

        let mut orchestrator = PluginOrchestrator::new(vec![native, bun, wasm]);
        orchestrator
            .load_from_config(loader.opencode().plugins.as_slice())
            .await?;

        tracing::info!(
            plugins = orchestrator.loaded_count(),
            theme = ?loader.tui().theme,
            "jerekode run — sidecar plugin host active"
        );

        orchestrator
            .dispatch_hook(jerekode_plugins::HookCall {
                hook: "tui.render".into(),
                payload: serde_json::json!({
                    "theme": loader.tui().theme,
                    "bootstrap": true
                }),
            })
            .await?;

        let _ = port.send(SidecarOutbound::Shutdown).await;
    }

    #[cfg(not(feature = "bun-sidecar"))]
    {
        let native = Arc::new(NativePluginHost::new());
        let wasm = Arc::new(WasmPluginHost::new());
        let mut orchestrator = PluginOrchestrator::new(vec![native, wasm]);
        orchestrator
            .load_from_config(loader.opencode().plugins.as_slice())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        tracing::info!(
            plugins = orchestrator.loaded_count(),
            theme = ?loader.tui().theme,
            "jerekode run — native-only build (no Bun sidecar)"
        );

        orchestrator
            .dispatch_hook(jerekode_plugins::HookCall {
                hook: "tui.render".into(),
                payload: serde_json::json!({
                    "theme": loader.tui().theme,
                    "bootstrap": true
                }),
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    #[cfg(feature = "bun-sidecar")]
    fn bun_available() -> bool {
        StdCommand::new("bun")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(feature = "bun-sidecar")]
    fn sidecar_entry() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecar/src/index.ts")
    }

    #[cfg(feature = "bun-sidecar")]
    #[tokio::test]
    async fn execute_boots_sidecar_plugin_host() {
        if !bun_available() {
            if std::env::var_os("CI").is_some() {
                panic!("execute_boots_sidecar_plugin_host requires bun on PATH");
            }
            eprintln!("skipping: bun unavailable");
            return;
        }

        let entry = sidecar_entry();
        assert!(
            entry.exists(),
            "sidecar entry missing at {}",
            entry.display()
        );

        let project = tempfile::tempdir().expect("temp project");
        let opencode = project.path().join(".opencode");
        fs::create_dir_all(&opencode).expect("mkdir .opencode");
        fs::write(
            opencode.join("tui.json"),
            serde_json::json!({
                "theme": "test",
                "sidecar": { "entry": entry.to_string_lossy() }
            })
            .to_string(),
        )
        .expect("write tui.json");

        execute(RunArgs {
            provider: None,
            model: None,
            project: Some(project.path().to_string_lossy().into_owned()),
        })
        .await
        .expect("run execute with bun sidecar");
    }
}
