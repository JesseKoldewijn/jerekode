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

    /// Attach to a running serve URL instead of in-process agent
    #[arg(long)]
    pub attach: Option<String>,

    /// File(s) to attach to the message
    #[arg(short = 'f', long = "file")]
    pub file: Vec<String>,

    /// Basic auth password when using `--attach`
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// Basic auth username when using `--attach`
    #[arg(short = 'u', long)]
    pub username: Option<String>,
}

pub async fn execute(args: RunArgs) -> anyhow::Result<ExitCode> {
    let mut message = args.message.join(" ").trim().to_string();
    for path in &args.file {
        let body = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("read --file {path}: {e}"))?;
        if !message.is_empty() {
            message.push_str(
                "

",
            );
        }
        message.push_str(&format!(
            "--- file: {path} ---
{body}"
        ));
    }
    if message.is_empty() {
        eprintln!("error: `jerekode run` requires a positional message");
        eprintln!("usage: jerekode run [OPTIONS] <MESSAGE>...");
        return Ok(ExitCode::from(2));
    }

    if let Some(url) = args.attach.clone() {
        return execute_attached(args, &url).await;
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

async fn execute_attached(args: RunArgs, url: &str) -> anyhow::Result<ExitCode> {
    let mut message = args.message.join(" ").trim().to_string();
    for path in &args.file {
        let body = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("read --file {path}: {e}"))?;
        if !message.is_empty() {
            message.push_str(
                "

",
            );
        }
        message.push_str(&format!(
            "--- file: {path} ---
{body}"
        ));
    }
    if message.is_empty() {
        eprintln!("error: `jerekode run --attach` requires a positional message");
        return Ok(ExitCode::from(2));
    }

    let password = args
        .password
        .or_else(|| env::var("OPENCODE_SERVER_PASSWORD").ok())
        .or_else(|| env::var("JEREKODE_SERVER_PASSWORD").ok());
    let username = args
        .username
        .or_else(|| env::var("OPENCODE_SERVER_USERNAME").ok())
        .unwrap_or_else(|| "opencode".into());

    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}/v2/sessions", url.trim_end_matches('/')))
        .json(&serde_json::json!({}));
    if let Some(pw) = password.as_ref() {
        req = req.basic_auth(&username, Some(pw));
    }
    let created = req.send().await?;
    if !created.status().is_success() {
        eprintln!("error: attach create session: {}", created.status());
        return Ok(ExitCode::from(1));
    }
    let body: serde_json::Value = created.json().await?;
    let session_id = body["session"]["id"].as_str().unwrap_or("");
    if session_id.is_empty() {
        eprintln!("error: attach response missing session id: {body}");
        return Ok(ExitCode::from(1));
    }

    let mut msg_req = client
        .post(format!(
            "{}/v2/sessions/{session_id}/messages",
            url.trim_end_matches('/')
        ))
        .json(&serde_json::json!({ "content": message }));
    if let Some(pw) = password.as_ref() {
        msg_req = msg_req.basic_auth(&username, Some(pw));
    }
    let resp = msg_req.send().await?;
    if !resp.status().is_success() {
        eprintln!("error: attach message: {}", resp.status());
        return Ok(ExitCode::from(1));
    }
    let msg_body: serde_json::Value = resp.json().await?;
    let text = msg_body["content"]
        .as_str()
        .or_else(|| msg_body["message"]["content"].as_str())
        .unwrap_or("");
    if args.format == "json" {
        println!("{msg_body}");
    } else {
        println!("{text}");
    }
    Ok(ExitCode::SUCCESS)
}
