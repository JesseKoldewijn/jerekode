mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "jerekode",
    about = "Jerekode — AI coding agent runtime",
    version,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve(commands::serve::ServeArgs),
    /// Run an interactive session (delegates to Bun sidecar in Phase 2)
    Run(commands::run::RunArgs),
    /// Print version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => commands::serve::execute(args).await?,
        Commands::Run(args) => commands::run::execute(args).await?,
        Commands::Version => commands::version::execute(),
    }

    Ok(())
}
