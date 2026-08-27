use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum SessionCommand {
    /// List sessions from a running `jerekode serve`
    List(SessionListArgs),
    /// Delete a session by id
    Delete(SessionDeleteArgs),
}

#[derive(Args, Debug)]
pub struct SessionListArgs {
    /// Base URL of the running server
    #[arg(long, default_value = "http://127.0.0.1:4096")]
    pub url: String,

    /// Output format: `table` or `json`
    #[arg(long, default_value = "table")]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct SessionDeleteArgs {
    /// Session id to delete
    pub id: String,

    /// Base URL of the running server
    #[arg(long, default_value = "http://127.0.0.1:4096")]
    pub url: String,
}

pub async fn execute(cmd: SessionCommand) -> anyhow::Result<()> {
    match cmd {
        SessionCommand::List(args) => list(args).await,
        SessionCommand::Delete(args) => delete(args).await,
    }
}

async fn list(args: SessionListArgs) -> anyhow::Result<()> {
    let base = args.url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/v2/sessions"))
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    let sessions = body
        .get("sessions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        _ => {
            println!("ID");
            println!("{}", "-".repeat(36));
            for s in sessions {
                if let Some(id) = s.as_str() {
                    println!("{id}");
                } else {
                    println!("{s}");
                }
            }
        }
    }
    Ok(())
}

async fn delete(args: SessionDeleteArgs) -> anyhow::Result<()> {
    let base = args.url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let resp = client
        .delete(format!("{base}/v2/sessions/{}", args.id))
        .send()
        .await?;
    if resp.status().is_success() || resp.status() == reqwest::StatusCode::NO_CONTENT {
        println!("deleted {}", args.id);
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("delete failed ({status}): {text}");
    }
}
