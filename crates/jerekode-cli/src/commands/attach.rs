use clap::Args;
use std::env;
use std::process::ExitCode;

use crate::commands::tui;

#[derive(Args, Debug)]
pub struct AttachArgs {
    /// Remote serve URL (e.g. http://127.0.0.1:4096)
    pub url: String,

    /// Working directory for the TUI
    #[arg(long)]
    pub dir: Option<String>,

    /// Basic auth password (defaults to OPENCODE_SERVER_PASSWORD / JEREKODE_SERVER_PASSWORD)
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// Basic auth username (defaults to OPENCODE_SERVER_USERNAME / opencode)
    #[arg(short = 'u', long)]
    pub username: Option<String>,
}

/// Attach the Bun TUI to a running `jerekode serve`.
pub async fn execute(args: AttachArgs) -> anyhow::Result<ExitCode> {
    if let Some(dir) = args.dir {
        env::set_current_dir(&dir).map_err(|e| anyhow::anyhow!("failed to chdir {}: {e}", dir))?;
    }
    // SAFETY: process-local env for child TUI / subsequent HTTP clients.
    unsafe {
        env::set_var("JEREKODE_SERVER_URL", &args.url);
        if let Some(pw) = args
            .password
            .or_else(|| env::var("OPENCODE_SERVER_PASSWORD").ok())
            .or_else(|| env::var("JEREKODE_SERVER_PASSWORD").ok())
        {
            env::set_var("OPENCODE_SERVER_PASSWORD", &pw);
            env::set_var("JEREKODE_SERVER_PASSWORD", &pw);
        }
        if let Some(user) = args
            .username
            .or_else(|| env::var("OPENCODE_SERVER_USERNAME").ok())
        {
            env::set_var("OPENCODE_SERVER_USERNAME", &user);
        }
    }
    tui::execute().await
}
