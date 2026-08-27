use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};
use jerekode_server::{RouterOptions, serve_with};
use std::env;

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Override bind host (`--hostname` is an OpenCode-compatible alias)
    #[arg(long, visible_alias = "hostname")]
    pub host: Option<String>,

    /// Override bind port
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Extra CORS origin (repeatable)
    #[arg(long = "cors", action = clap::ArgAction::Append)]
    pub cors: Vec<String>,

    /// Override default provider
    #[arg(long)]
    pub provider: Option<String>,

    /// Override default model
    #[arg(long)]
    pub model: Option<String>,

    /// Project root for config discovery (defaults to current directory)
    #[arg(long)]
    pub project: Option<String>,
}

pub async fn execute(args: ServeArgs) -> anyhow::Result<()> {
    let project = args
        .project
        .map(Into::into)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));

    let cli = CliOverrides {
        host: args.host,
        port: args.port,
        provider: args.provider,
        model: args.model,
    };

    let loader = ConfigLoader::load_discovered(&project, &cli)?;
    tracing::info!(layers = ?loader.loaded_layers(), "loaded config");

    let opts = RouterOptions {
        cors_origins: args.cors,
        basic_auth: Default::default(),
    };
    serve_with(loader.opencode(), opts).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::time::Duration;

    fn pick_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    #[tokio::test]
    async fn execute_binds_and_answers_health() {
        let port = pick_port();
        let project = tempfile::tempdir().expect("temp project");
        let args = ServeArgs {
            host: Some("127.0.0.1".into()),
            port: Some(port),
            cors: Vec::new(),
            provider: None,
            model: None,
            project: Some(project.path().to_string_lossy().into_owned()),
        };

        let handle = tokio::spawn(async move { execute(args).await });
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/health");
        let mut healthy = false;
        for _ in 0..50 {
            if let Ok(resp) = client.get(&url).send().await
                && resp.status().is_success()
            {
                healthy = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        handle.abort();
        let _ = handle.await;
        assert!(
            healthy,
            "serve execute did not become healthy on port {port}"
        );
    }

    #[tokio::test]
    async fn execute_accepts_hostname_via_host_field() {
        let port = pick_port();
        let project = tempfile::tempdir().expect("temp project");
        // `hostname` is a clap alias; the field is still `host`.
        let args = ServeArgs {
            host: Some("127.0.0.1".into()),
            port: Some(port),
            cors: vec!["http://localhost:3000".into()],
            provider: None,
            model: None,
            project: Some(project.path().to_string_lossy().into_owned()),
        };
        let handle = tokio::spawn(async move { execute(args).await });
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/health");
        let mut healthy = false;
        for _ in 0..50 {
            if let Ok(resp) = client.get(&url).send().await
                && resp.status().is_success()
            {
                healthy = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        handle.abort();
        let _ = handle.await;
        assert!(healthy);
    }
}
