use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};
use jerekode_server;
use std::env;

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Override bind host
    #[arg(long)]
    pub host: Option<String>,

    /// Override bind port
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Override default provider
    #[arg(long)]
    pub provider: Option<String>,

    /// Override default model
    #[arg(long)]
    pub model: Option<String>,

    /// Project root for config discovery (defaults to current directory)
    #[arg(long)]
    pub project: Option<String>,
}

pub async fn execute(args: ServeArgs) -> anyhow::Result<()> {
    let project = args
        .project
        .map(Into::into)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));

    let cli = CliOverrides {
        host: args.host,
        port: args.port,
        provider: args.provider,
        model: args.model,
    };

    let loader = ConfigLoader::load_discovered(&project, &cli)?;
    tracing::info!(layers = ?loader.loaded_layers(), "loaded config");

    jerekode_server::serve(loader.opencode()).await?;
    Ok(())
}
