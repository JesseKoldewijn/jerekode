use clap::Subcommand;
use jerekode_config::{CliOverrides, ConfigLoader};
use std::env;
use std::process::ExitCode;

#[derive(Subcommand, Debug)]
pub enum DbCommand {
    /// Print the session database path
    Path,
}

pub async fn execute(cmd: DbCommand) -> anyhow::Result<ExitCode> {
    match cmd {
        DbCommand::Path => {
            let project = env::current_dir()?;
            let loader = ConfigLoader::load_discovered(&project, &CliOverrides::default())?;
            match &loader.opencode().session_db {
                Some(p) => println!("{p}"),
                None => {
                    let fallback = dirs_data().join("jerekode").join("session.db");
                    println!("{}", fallback.display());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn dirs_data() -> std::path::PathBuf {
    if let Ok(xdg) = env::var("XDG_DATA_HOME")
        && !xdg.is_empty()
    {
        return std::path::PathBuf::from(xdg);
    }
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".local")
        .join("share")
}
