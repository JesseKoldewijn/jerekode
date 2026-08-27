//! HTTP server with a pluggable v1/v2 adapter layer.
//!
//! All external HTTP shapes normalize into `adapters::normalized` types as
//! early as possible. Handlers operate only on normalized requests, making v1
//! deprecation a matter of removing the v1 adapter rather than touching core logic.

pub mod adapters;
pub mod error;
pub mod extensions;
pub mod handlers;
pub mod persistence;
pub mod policy;
pub mod router;
pub mod session_store;
pub mod sse;
pub mod state;
pub mod tools;

use jerekode_config::{OpenCodeConfig, PluginEntry};
#[cfg(feature = "bun-sidecar")]
use jerekode_plugins::{BunPluginHost, BunProcessSidecarPort};
use jerekode_plugins::{NativePluginHost, PluginHost, PluginOrchestrator, WasmPluginHost};
use std::net::SocketAddr;
use std::sync::Arc;

pub use error::{ServerError, ServerResult};
pub use persistence::SqliteSessionStore;
pub use router::build_router;
pub use session_store::{SessionStore, SessionStorePort};
pub use state::AppState;

/// Start the HTTP server on a specific address (for tests — stub providers).
pub async fn serve_on(addr: SocketAddr, config: &OpenCodeConfig) -> ServerResult<()> {
    let state = if let Some(path) = config.session_db.as_ref() {
        AppState::with_sqlite(config, path).map_err(ServerError::Serve)?
    } else {
        AppState::new(config)
    };
    let app = build_router(state);

    tracing::info!(%addr, "jerekode server listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ServerError::Bind(addr.to_string(), e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Serve(e.to_string()))?;

    Ok(())
}

/// Start the HTTP server using config host/port.
pub async fn serve(config: &OpenCodeConfig) -> ServerResult<()> {
    let host = config.host.as_deref().unwrap_or("127.0.0.1");
    let port = config.port.unwrap_or(4096);
    let addr: SocketAddr = format!("{host}:{port}").parse().map_err(|e| {
        ServerError::Bind(
            format!("{host}:{port}"),
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        )
    })?;
    let state = build_app_state(config).await?;
    let app = build_router(state);

    tracing::info!(%addr, "jerekode server listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ServerError::Bind(addr.to_string(), e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Serve(e.to_string()))?;

    Ok(())
}

fn build_base_state(config: &OpenCodeConfig) -> ServerResult<AppState> {
    if std::env::var("JEREKO_USE_STUB_PROVIDERS").is_ok() {
        if let Some(path) = config.session_db.as_ref() {
            return AppState::with_sqlite(config, path).map_err(ServerError::Serve);
        }
        return Ok(AppState::new(config));
    }
    AppState::production(config).map_err(ServerError::Serve)
}

fn needs_bun_host(entries: &[PluginEntry]) -> bool {
    entries
        .iter()
        .any(|e| matches!(e, PluginEntry::Bun(_) | PluginEntry::Named { .. }))
}

/// Build app state and load configured plugins into a `PluginOrchestrator`.
pub async fn build_app_state(config: &OpenCodeConfig) -> ServerResult<AppState> {
    let base = build_base_state(config)?;
    if config.plugins.is_empty() {
        return Ok(base);
    }

    let mut hosts: Vec<Arc<dyn PluginHost>> = Vec::new();
    if needs_bun_host(&config.plugins) {
        #[cfg(feature = "bun-sidecar")]
        {
            let entry = std::env::var("JEREKO_SIDECAR_ENTRY")
                .unwrap_or_else(|_| "sidecar/src/index.ts".into());
            let process = BunProcessSidecarPort::spawn(entry)
                .await
                .map_err(|e| ServerError::Serve(e.to_string()))?;
            process
                .wait_startup_ready()
                .await
                .map_err(|e| ServerError::Serve(e.to_string()))?;
            hosts.push(Arc::new(BunPluginHost::new(process)));
        }
        #[cfg(not(feature = "bun-sidecar"))]
        {
            return Err(ServerError::Serve(format!(
                "Bun/TS plugins are configured, but {}",
                jerekode_plugins::BUN_SIDECAR_UNAVAILABLE_MSG
            )));
        }
    }
    hosts.push(Arc::new(NativePluginHost::new()));
    hosts.push(Arc::new(WasmPluginHost::default()));

    let mut orch = PluginOrchestrator::new(hosts);
    orch.load_from_config(&config.plugins)
        .await
        .map_err(|e| ServerError::Serve(e.to_string()))?;
    Ok(base.with_plugins(orch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    #[cfg(feature = "bun-sidecar")]
    fn bun_available() -> bool {
        StdCommand::new("bun")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(feature = "bun-sidecar")]
    fn sidecar_entry() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecar/src/index.ts")
    }

    #[cfg(feature = "bun-sidecar")]
    #[tokio::test]
    async fn build_app_state_loads_bun_host_when_plugins_need_it() {
        if !bun_available() {
            if std::env::var_os("CI").is_some() {
                panic!("build_app_state_loads_bun_host_when_plugins_need_it requires bun");
            }
            eprintln!("skipping: bun unavailable");
            return;
        }

        let entry = sidecar_entry();
        assert!(
            entry.exists(),
            "sidecar entry missing at {}",
            entry.display()
        );
        // SAFETY: test-only env mutation; restored below.
        let previous = std::env::var_os("JEREKO_SIDECAR_ENTRY");
        unsafe {
            std::env::set_var("JEREKO_SIDECAR_ENTRY", entry.as_os_str());
            std::env::set_var("JEREKO_USE_STUB_PROVIDERS", "1");
        }

        let config = OpenCodeConfig {
            plugins: vec![PluginEntry::Bun("@acme/server-plugin".into())],
            ..Default::default()
        };
        let result = build_app_state(&config).await;

        match previous {
            Some(v) => unsafe { std::env::set_var("JEREKO_SIDECAR_ENTRY", v) },
            None => unsafe { std::env::remove_var("JEREKO_SIDECAR_ENTRY") },
        }
        unsafe {
            std::env::remove_var("JEREKO_USE_STUB_PROVIDERS");
        }

        let state = result.expect("build_app_state with bun plugins");
        assert!(
            state.ctx.plugins.is_some(),
            "plugins orchestrator must be attached"
        );
    }
}
