use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;

use crate::{repo::pin::DBPin, routes::pin_type, state::AppState};

async fn get_all_pins(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pins: Vec<DBPin> = match state.pin_repo.get_all_pin(&state.pool).await {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    (StatusCode::OK, Json(pins)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_pins))
        .nest("/type", pin_type::routes())
}
