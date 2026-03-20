use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use bytes::Bytes;
use reqwest::StatusCode;

use crate::state::AppState;

async fn get_file_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    let output: Bytes = match state.storage_utils.fetch_public_file(&id.to_string()).await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    (StatusCode::OK, output).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/{id}", get(get_file_by_id))
}
