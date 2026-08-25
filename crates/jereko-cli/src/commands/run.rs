use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Provider id override
    #[arg(long)]
    pub provider: Option<String>,

    /// Model id override
    #[arg(long)]
    pub model: Option<String>,
}

pub async fn execute(args: RunArgs) -> anyhow::Result<()> {
    // TODO(phase-2): spawn Bun sidecar and attach TUI plugin host
    tracing::info!(
        provider = ?args.provider,
        model = ?args.model,
        "run command stub — sidecar integration pending"
    );
    eprintln!("jereko run: sidecar TUI not yet implemented (Phase 2)");
    Ok(())
}
