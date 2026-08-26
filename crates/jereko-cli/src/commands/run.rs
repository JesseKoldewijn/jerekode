use clap::Args;
use jereko_config::{CliOverrides, ConfigLoader};
use jereko_plugins::{
    BunPluginHost, BunProcessSidecarPort, NativePluginHost, PluginOrchestrator, SidecarOutbound,
    SidecarPort, WasmPluginHost,
};
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

    let sidecar_entry = loader
        .tui()
        .sidecar
        .as_ref()
        .and_then(|s| s.entry.clone())
        .unwrap_or_else(|| "sidecar/src/index.ts".into());

    let port: Arc<dyn SidecarPort> = BunProcessSidecarPort::spawn(sidecar_entry).await?;
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
        "jereko run — sidecar plugin host active"
    );

    // TUI render bootstrap stub (Phase 3)
    orchestrator
        .dispatch_hook(jereko_plugins::HookCall {
            hook: "tui.render".into(),
            payload: serde_json::json!({
                "theme": loader.tui().theme,
                "bootstrap": true
            }),
        })
        .await?;

    jereko_plugins::run_sidecar_loop(port.as_ref()).await?;

    // Graceful teardown when the run loop exits (e.g. after ready handshake in short runs).
    let _ = port.send(SidecarOutbound::Shutdown).await;

    Ok(())
}
