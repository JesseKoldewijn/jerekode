use clap::Subcommand;
use std::process::ExitCode;

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// List agents
    List,
}

pub async fn execute(cmd: AgentCommand) -> anyhow::Result<ExitCode> {
    match cmd {
        AgentCommand::List => {
            println!("(no custom agents)");
            Ok(ExitCode::SUCCESS)
        }
    }
}
