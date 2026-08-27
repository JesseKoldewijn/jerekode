mod commands;
mod util;

use clap::{ArgAction, Parser, Subcommand};
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
    /// Print version
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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
    if cli.version {
        // OpenCode-compatible `-v` / `--version` (clap default short is `-V`).
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    match cli.command {
        None => commands::tui::execute().await,
        Some(Commands::Serve(args)) => {
            commands::serve::execute(args).await?;
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Run(args)) => commands::run::execute(args).await,
        Some(Commands::Models(args)) => {
            commands::models::execute(args).await?;
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Session { command }) => {
            commands::session::execute(command).await?;
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Version) => {
            commands::version::execute();
            Ok(ExitCode::SUCCESS)
        }
    }
}
