use crate::adapters::v2::{
    V2CreateSessionRequest, V2ErrorResponse, V2SendMessageRequest, denormalize_create_session,
    denormalize_send_message, normalize_create_session,
};
use crate::handlers::HandlerError;
use crate::sse::format_completion_sse;
use crate::state::AppState;
use crate::tools::{ToolCall, ToolResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions/{id}", get(get_session))
        .route("/sessions/{id}/messages", post(send_message))
        .route("/sessions/{id}/messages/stream", post(send_message_stream))
        .route("/providers", get(list_providers))
        .route("/tools/execute", post(execute_tool))
}

async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<V2CreateSessionRequest>,
) -> impl IntoResponse {
    let normalized_req = normalize_create_session(req);
    match state.ctx.create_session(normalized_req) {
        Ok(resp) => {
            let v2_resp = denormalize_create_session(resp);
            (StatusCode::CREATED, Json(v2_resp)).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn get_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.get_session(&id) {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => map_error(e),
    }
}

async fn send_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<V2SendMessageRequest>,
) -> impl IntoResponse {
    match state.ctx.send_message(&id, req.content).await {
        Ok(resp) => {
            let v2 = denormalize_send_message(resp);
            (StatusCode::OK, Json(v2)).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn send_message_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<V2SendMessageRequest>,
) -> impl IntoResponse {
    match state.ctx.send_message_stream(&id, req.content).await {
        Ok(result) => {
            let body = format_completion_sse(&result.chunks, &result.assistant_message);
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/event-stream"),
            );
            headers.insert(
                axum::http::header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache"),
            );
            (StatusCode::OK, headers, body).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let resp = state.ctx.list_providers();
    (StatusCode::OK, Json(resp)).into_response()
}

async fn execute_tool(
    State(state): State<AppState>,
    Json(call): Json<ToolCall>,
) -> impl IntoResponse {
    let result: ToolResult = state.ctx.execute_tool(call);
    let status = if result.ok {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (status, Json(result)).into_response()
}

fn map_error(err: HandlerError) -> axum::response::Response {
    let (status, code, message) = match err {
        HandlerError::SessionNotFound(id) => (StatusCode::NOT_FOUND, "not_found", id),
        HandlerError::ProviderNotFound(id) => (
            StatusCode::BAD_REQUEST,
            "invalid_provider",
            format!("unknown provider: {id}"),
        ),
        HandlerError::Provider(msg) => (StatusCode::BAD_GATEWAY, "provider_error", msg),
        HandlerError::Tool(msg) => (StatusCode::BAD_REQUEST, "tool_error", msg),
    };
    (
        status,
        Json(V2ErrorResponse {
            code: code.into(),
            message,
        }),
    )
        .into_response()
}
