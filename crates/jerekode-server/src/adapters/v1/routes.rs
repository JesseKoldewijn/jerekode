use crate::adapters::normalized;
use crate::adapters::v1::{
    V1CreateSessionRequest, V1ErrorResponse, V1SendMessageRequest, denormalize_create_session,
    denormalize_list_providers, denormalize_send_message, normalize_create_session,
    normalize_send_message,
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
        .route("/session", get(list_sessions).post(create_session))
        .route("/session/{id}", get(get_session).delete(delete_session))
        .route(
            "/session/{id}/message",
            get(list_messages).post(send_message),
        )
        .route("/session/{id}/message/stream", post(send_message_stream))
        .route("/providers", get(list_providers))
        .route("/tools/execute", post(execute_tool))
}

async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<V1CreateSessionRequest>,
) -> impl IntoResponse {
    let normalized_req = normalize_create_session(req);
    match state.ctx.create_session(normalized_req) {
        Ok(resp) => {
            let v1_resp = denormalize_create_session(resp);
            (StatusCode::CREATED, Json(v1_resp)).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn get_session(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.get_session(&id) {
        Ok(resp) => {
            let v1 = denormalize_create_session(normalized::CreateSessionResponse {
                session: resp.session,
            });
            (StatusCode::OK, Json(v1)).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn send_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<V1SendMessageRequest>,
) -> impl IntoResponse {
    let content = normalize_send_message(req);
    match state.ctx.send_message(&id, content).await {
        Ok(resp) => {
            let v1 = denormalize_send_message(resp);
            (StatusCode::OK, Json(v1)).into_response()
        }
        Err(e) => map_error(e),
    }
}

async fn send_message_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<V1SendMessageRequest>,
) -> impl IntoResponse {
    let content = normalize_send_message(req);
    match state.ctx.send_message_stream(&id, content).await {
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
    let v1 = denormalize_list_providers(resp);
    (StatusCode::OK, Json(v1)).into_response()
}

async fn execute_tool(
    State(state): State<AppState>,
    Json(call): Json<ToolCall>,
) -> impl IntoResponse {
    let result: ToolResult = state.ctx.execute_tool(call).await;
    let status = if result.ok {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (status, Json(result)).into_response()
}

async fn list_sessions(State(state): State<AppState>) -> impl IntoResponse {
    let ids = state.ctx.list_sessions();
    (StatusCode::OK, Json(serde_json::json!({ "sessions": ids }))).into_response()
}

async fn list_messages(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.list_messages(&id) {
        Ok(messages) => (
            StatusCode::OK,
            Json(serde_json::json!({ "messages": messages })),
        )
            .into_response(),
        Err(e) => map_error(e),
    }
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.ctx.delete_session(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e),
    }
}

fn map_error(err: HandlerError) -> axum::response::Response {
    let (status, message) = match err {
        HandlerError::SessionNotFound(id) => (StatusCode::NOT_FOUND, id),
        HandlerError::ProviderNotFound(id) => {
            (StatusCode::BAD_REQUEST, format!("unknown provider: {id}"))
        }
        HandlerError::Provider(msg) => (StatusCode::BAD_GATEWAY, msg),
        HandlerError::Tool(msg) => (StatusCode::BAD_REQUEST, msg),
    };
    (status, Json(V1ErrorResponse { error: message })).into_response()
}
