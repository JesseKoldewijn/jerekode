use crate::adapters;
use crate::extensions;
use crate::state::AppState;
use axum::{Json, Router, routing::get};

/// Build the Axum router with v1 and v2 adapter routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/extensions/mcp", get(mcp_status))
        .route("/extensions/lsp", get(lsp_status))
        .route("/extensions/pty", get(pty_status))
        .nest("/v1", adapters::v1::router())
        .nest("/v2", adapters::v2::router())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn mcp_status() -> Json<extensions::McpStubResponse> {
    Json(extensions::mcp_status_stub())
}

async fn lsp_status() -> Json<extensions::LspStubResponse> {
    Json(extensions::lsp_status_stub())
}

async fn pty_status() -> Json<extensions::PtyStubResponse> {
    Json(extensions::pty_status_stub())
}
