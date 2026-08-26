use crate::adapters::normalized;
use crate::adapters::v1::{
    V1CreateSessionRequest, V1ErrorResponse, V1SendMessageRequest, denormalize_create_session,
    denormalize_list_providers, denormalize_send_message, normalize_create_session,
    normalize_send_message,
};
use crate::handlers::HandlerError;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/session", post(create_session))
        .route("/session/{id}", get(get_session))
        .route("/session/{id}/message", post(send_message))
        .route("/providers", get(list_providers))
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

async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let resp = state.ctx.list_providers();
    let v1 = denormalize_list_providers(resp);
    (StatusCode::OK, Json(v1)).into_response()
}

fn map_error(err: HandlerError) -> axum::response::Response {
    let (status, message) = match err {
        HandlerError::SessionNotFound(id) => (StatusCode::NOT_FOUND, id),
        HandlerError::ProviderNotFound(id) => {
            (StatusCode::BAD_REQUEST, format!("unknown provider: {id}"))
        }
        HandlerError::Provider(msg) => (StatusCode::BAD_GATEWAY, msg),
    };
    (status, Json(V1ErrorResponse { error: message })).into_response()
}
