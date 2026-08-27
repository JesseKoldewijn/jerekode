use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};
use jerekode_plugins::{NativePluginHost, PluginOrchestrator, WasmPluginHost};
#[cfg(feature = "bun-sidecar")]
use jerekode_plugins::{BunPluginHost, BunProcessSidecarPort, SidecarOutbound, SidecarPort};
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
