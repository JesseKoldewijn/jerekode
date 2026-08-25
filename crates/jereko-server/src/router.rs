use crate::adapters;
use crate::state::AppState;
use axum::{routing::get, Router};

/// Build the Axum router with v1 and v2 adapter routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/v1", adapters::v1::router())
        .nest("/v2", adapters::v2::router())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
