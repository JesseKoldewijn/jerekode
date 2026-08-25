use clap::Args;
use jereko_config::ConfigLoader;
use jereko_server;

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Override bind host
    #[arg(long)]
    pub host: Option<String>,

    /// Override bind port
    #[arg(short, long)]
    pub port: Option<u16>,
}

pub async fn execute(args: ServeArgs) -> anyhow::Result<()> {
    let loader = ConfigLoader::new();

    // TODO(phase-1): load discovered config paths with proper precedence
    let config = loader.opencode().clone();
    let mut config = config;

    if let Some(host) = args.host {
        config.host = Some(host);
    }
    if let Some(port) = args.port {
        config.port = Some(port);
    }

    jereko_server::serve(&config).await?;
    Ok(())
}
