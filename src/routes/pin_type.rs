use std::sync::Arc;

use crate::{repo::pin_types::DBPinType, state::AppState};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use reqwest::StatusCode;

async fn get_all_pin_types(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pin_types: Vec<DBPinType> = match state.pin_type_repo.get_all_pin_types(&mut tx).await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pin_types)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_pin_types))
}
