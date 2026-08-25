//! HTTP server with a pluggable v1/v2 adapter layer.
//!
//! All external HTTP shapes normalize into `adapters::normalized` types as
//! early as possible. Handlers operate only on normalized requests, making v1
//! deprecation a matter of removing the v1 adapter rather than touching core logic.

pub mod adapters;
pub mod error;
pub mod router;
pub mod state;

pub use error::{ServerError, ServerResult};
pub use router::build_router;
pub use state::AppState;

use jereko_config::OpenCodeConfig;
use std::net::SocketAddr;

/// Start the HTTP server (stub — binds and serves health endpoints).
pub async fn serve(config: &OpenCodeConfig) -> ServerResult<()> {
    let host = config.host.as_deref().unwrap_or("127.0.0.1");
    let port = config.port.unwrap_or(4096);
    let addr: SocketAddr = format!("{host}:{port}").parse().map_err(|e| {
        ServerError::Bind(
            format!("{host}:{port}"),
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        )
    })?;

    let state = AppState::default();
    let app = build_router(state);

    tracing::info!(%addr, "jereko server listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ServerError::Bind(addr.to_string(), e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Serve(e.to_string()))?;

    Ok(())
}
