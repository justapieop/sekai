use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;

async fn prompt(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let prompt: String = match query.get("prompt") {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(s) => s.to_string(),
    };
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().without_v07_checks()
}
