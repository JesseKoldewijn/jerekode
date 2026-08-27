use clap::Subcommand;
use jerekode_config::{CliOverrides, ConfigLoader};
use std::env;
use std::process::ExitCode;

#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// List MCP servers from config
    #[command(visible_alias = "ls")]
    List,
}

pub async fn execute(cmd: McpCommand) -> anyhow::Result<ExitCode> {
    match cmd {
        McpCommand::List => {
            let project = env::current_dir()?;
            let loader = ConfigLoader::load_discovered(&project, &CliOverrides::default())?;
            let cfg = loader.opencode();
            // MCP entries are not yet a first-class config field; report honestly.
            let _ = cfg;
            println!("(no MCP servers configured)");
            Ok(ExitCode::SUCCESS)
        }
    }
}
