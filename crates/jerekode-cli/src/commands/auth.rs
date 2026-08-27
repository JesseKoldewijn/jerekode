use clap::{Args, Subcommand};
use jerekode_config::{
    AuthCredential, import_opencode_into, jerekode_auth_path, load_store, save_store,
};
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Log in to a provider (writes jerekode auth store only)
    Login(LoginArgs),
    /// List authenticated providers
    #[command(visible_alias = "ls")]
    List,
    /// Remove a provider from the jerekode auth store
    Logout(LogoutArgs),
    /// Import credentials from OpenCode into the jerekode store
    Import,
}

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Provider id (e.g. openai, anthropic)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,
    /// Login method label
    #[arg(short = 'm', long)]
    pub method: Option<String>,
    /// API key (non-interactive; otherwise prompted or `JEREKODE_API_KEY`)
    #[arg(long)]
    pub api_key: Option<String>,
}

#[derive(Args, Debug)]
pub struct LogoutArgs {
    /// Provider id to remove
    pub provider: String,
}

pub async fn execute(cmd: AuthCommand) -> anyhow::Result<ExitCode> {
    match cmd {
        AuthCommand::List => {
            let path = jerekode_auth_path();
            let store = load_store(&path)?;
            if store.providers.is_empty() {
                println!("(no credentials in {})", path.display());
                return Ok(ExitCode::SUCCESS);
            }
            for id in store.list_ids() {
                let method = store.providers[&id].method.as_deref().unwrap_or("api");
                println!("{id}\t{method}");
            }
            Ok(ExitCode::SUCCESS)
        }
        AuthCommand::Login(args) => {
            let provider = match args.provider {
                Some(p) => p,
                None => {
                    eprint!("provider: ");
                    io::stderr().flush()?;
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    let p = buf.trim().to_string();
                    if p.is_empty() {
                        eprintln!("error: provider is required");
                        return Ok(ExitCode::from(2));
                    }
                    p
                }
            };
            let api_key = match args
                .api_key
                .or_else(|| std::env::var("JEREKODE_API_KEY").ok())
            {
                Some(k) => k,
                None => {
                    eprint!("api key: ");
                    io::stderr().flush()?;
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    let k = buf.trim().to_string();
                    if k.is_empty() {
                        eprintln!("error: api key is required");
                        return Ok(ExitCode::from(2));
                    }
                    k
                }
            };
            let path = jerekode_auth_path();
            let mut store = load_store(&path)?;
            store.upsert(
                &provider,
                AuthCredential {
                    api_key,
                    method: args.method.or_else(|| Some("api".into())),
                },
            );
            save_store(&path, &store)?;
            println!("logged in: {provider} ({})", path.display());
            Ok(ExitCode::SUCCESS)
        }
        AuthCommand::Logout(args) => {
            let path = jerekode_auth_path();
            let mut store = load_store(&path)?;
            if store.remove(&args.provider) {
                save_store(&path, &store)?;
                println!("logged out: {}", args.provider);
                Ok(ExitCode::SUCCESS)
            } else {
                eprintln!("error: provider not found: {}", args.provider);
                Ok(ExitCode::from(1))
            }
        }
        AuthCommand::Import => {
            let path = jerekode_auth_path();
            let (store, source) = import_opencode_into(&path)?;
            match source {
                Some(src) => {
                    println!(
                        "imported {} provider(s) from {} → {}",
                        store.providers.len(),
                        src.display(),
                        path.display()
                    );
                }
                None => {
                    println!(
                        "no OpenCode auth.json found; jerekode store unchanged ({})",
                        path.display()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
