mod commands;
mod util;

use clap::{Parser, Subcommand};
use std::process::ExitCode;
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
    /// Run a one-shot agent prompt (non-interactive)
    Run(commands::run::RunArgs),
    /// List available models (`provider/model`)
    Models(commands::models::ModelsArgs),
    /// Session management (thin HTTP client)
    Session {
        #[command(subcommand)]
        command: commands::session::SessionCommand,
    },
    /// Print version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => {
            commands::serve::execute(args).await?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Run(args) => commands::run::execute(args).await,
        Commands::Models(args) => {
            commands::models::execute(args).await?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Session { command } => {
            commands::session::execute(command).await?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Version => {
            commands::version::execute();
            Ok(ExitCode::SUCCESS)
        }
    }
}
