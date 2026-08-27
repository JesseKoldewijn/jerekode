use crate::extensions::{self, LspHoverResult, McpToolResult, PtyIoResult};
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Deserialize;

/// Build the Axum router with v1 and v2 adapter routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/extensions/mcp", get(mcp_status))
        .route("/extensions/mcp/call", post(mcp_call))
        .route("/extensions/lsp", get(lsp_status))
        .route("/extensions/lsp/initialize", post(lsp_initialize))
        .route("/extensions/lsp/hover", post(lsp_hover))
        .route("/extensions/pty", get(pty_status))
        .route("/extensions/pty/spawn", post(pty_spawn))
        .route("/extensions/pty/write", post(pty_write))
        .route("/extensions/pty/read", post(pty_read))
        .nest("/v1", crate::adapters::v1::router())
        .nest("/v2", crate::adapters::v2::router())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn mcp_status(State(state): State<AppState>) -> Json<extensions::McpStatus> {
    Json(state.extensions.mcp.status())
}

#[derive(Debug, Deserialize)]
struct McpCallRequest {
    tool: String,
    #[serde(default)]
    args: serde_json::Value,
}

async fn mcp_call(
    State(state): State<AppState>,
    Json(req): Json<McpCallRequest>,
) -> Json<McpToolResult> {
    Json(state.extensions.mcp.call_tool(&req.tool, req.args))
}

async fn lsp_status(State(state): State<AppState>) -> Json<extensions::LspStatus> {
    Json(state.extensions.lsp.status())
}

#[derive(Debug, Deserialize)]
struct LspInitRequest {
    #[serde(default = "default_root")]
    root_uri: String,
}

fn default_root() -> String {
    "file:///tmp/jerekode".into()
}

async fn lsp_initialize(
    State(state): State<AppState>,
    Json(req): Json<LspInitRequest>,
) -> Json<extensions::LspStatus> {
    Json(state.extensions.lsp.initialize(&req.root_uri))
}

#[derive(Debug, Deserialize)]
struct LspHoverRequest {
    uri: String,
    line: u32,
    character: u32,
    #[serde(default)]
    text: Option<String>,
}

async fn lsp_hover(
    State(state): State<AppState>,
    Json(req): Json<LspHoverRequest>,
) -> Result<Json<LspHoverResult>, (axum::http::StatusCode, String)> {
    if let Some(text) = &req.text {
        state.extensions.lsp.open_document(&req.uri, text);
    }
    state
        .extensions
        .lsp
        .hover(&req.uri, req.line, req.character)
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e))
}

async fn pty_status(State(state): State<AppState>) -> Json<extensions::PtyStatus> {
    Json(state.extensions.pty.status())
}

#[derive(Debug, Deserialize)]
struct PtySpawnRequest {
    session_id: String,
    #[serde(default = "default_shell")]
    command: String,
}

fn default_shell() -> String {
    "echo jerekode-pty".into()
}

async fn pty_spawn(
    State(state): State<AppState>,
    Json(req): Json<PtySpawnRequest>,
) -> Json<serde_json::Value> {
    let id = state.extensions.pty.spawn(req.session_id, req.command);
    Json(serde_json::json!({ "session_id": id, "ok": true }))
}

#[derive(Debug, Deserialize)]
struct PtyWriteRequest {
    session_id: String,
    data: String,
}

async fn pty_write(
    State(state): State<AppState>,
    Json(req): Json<PtyWriteRequest>,
) -> Json<PtyIoResult> {
    Json(state.extensions.pty.write(&req.session_id, &req.data))
}

#[derive(Debug, Deserialize)]
struct PtyReadRequest {
    session_id: String,
}

async fn pty_read(
    State(state): State<AppState>,
    Json(req): Json<PtyReadRequest>,
) -> Json<PtyIoResult> {
    Json(state.extensions.pty.read(&req.session_id))
}
