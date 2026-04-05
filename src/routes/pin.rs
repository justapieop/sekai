use std::sync::Arc;

use crate::{repo::pin::DBPin, routes::pin_type, state::AppState};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;

async fn get_all_pins(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pins: Vec<DBPin> = match state.pin_repo.get_all_pin(&mut tx).await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pins)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_pins))
        .nest("/type", pin_type::routes())
}
