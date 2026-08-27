use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};
use jerekode_server::{AgentRunRequest, AppState};
use std::env;
use std::process::ExitCode;

use crate::util::resolve_provider_model;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Prompt message (one-shot). Multiple words are joined with spaces.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub message: Vec<String>,

    /// Provider id override
    #[arg(long)]
    pub provider: Option<String>,

    /// Model id, or `provider/model` (OpenCode form)
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Output format: `default` (plain text) or `json`
    #[arg(long, default_value = "default")]
    pub format: String,

    /// Project root for config discovery
    #[arg(long)]
    pub project: Option<String>,
}

pub async fn execute(args: RunArgs) -> anyhow::Result<ExitCode> {
    let message = args.message.join(" ").trim().to_string();
    if message.is_empty() {
        eprintln!("error: `jerekode run` requires a positional message");
        eprintln!("usage: jerekode run [OPTIONS] <MESSAGE>...");
        return Ok(ExitCode::from(2));
    }

    let project = args
        .project
        .map(Into::into)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));

    let (provider, model) = resolve_provider_model(args.provider, args.model);
    let cli = CliOverrides {
        provider: provider.clone(),
        model: model.clone(),
        ..Default::default()
    };
    let loader = ConfigLoader::load_discovered(&project, &cli)?;
    let config = loader.opencode().clone();

    // Force stub providers in tests / offline; otherwise production registry.
    // `AppState::new` always uses stubs; `production` uses real HTTP providers.
    let state = if std::env::var("JEREKO_USE_STUB_PROVIDERS").is_ok() {
        AppState::new(&config)
    } else {
        AppState::production(&config).map_err(|e| anyhow::anyhow!(e))?
    };

    let agent = state.agent_loop();
    match agent
        .run(AgentRunRequest {
            message,
            session_id: None,
            provider_id: provider.or_else(|| config.provider.clone()),
            model: model.or_else(|| config.model.clone()),
            max_turns: Some(8),
        })
        .await
    {
        Ok(result) => {
            match args.format.as_str() {
                "json" => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "session_id": result.session_id,
                            "content": result.final_text,
                            "turns": result.turns,
                        })
                    );
                }
                _ => {
                    println!("{}", result.final_text);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("error: {e}");
            Ok(ExitCode::from(1))
        }
    }
}
