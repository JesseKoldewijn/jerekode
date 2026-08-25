use crate::adapters::normalized;
use crate::adapters::v1::{
    denormalize_create_session, normalize_create_session, V1CreateSessionRequest,
};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/session", post(create_session))
}

async fn create_session(
    State(_state): State<AppState>,
    Json(req): Json<V1CreateSessionRequest>,
) -> impl IntoResponse {
    let normalized_req = normalize_create_session(req);
    let session = normalized::new_session(normalized_req.provider_id);
    let normalized_resp = normalized::CreateSessionResponse { session };
    let v1_resp = denormalize_create_session(normalized_resp);
    (StatusCode::CREATED, Json(v1_resp))
}
