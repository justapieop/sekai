use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;

use crate::{repo::pin_types::DBPinType, state::AppState};

async fn get_all_pin_types(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pin_types: Vec<DBPinType> = match state.pin_type_repo.get_all_pin_types(&state.pool).await {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    (StatusCode::OK, Json(pin_types)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_pin_types))
}
