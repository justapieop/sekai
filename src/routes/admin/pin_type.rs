use std::sync::Arc;

use crate::{repo::pin_types::DBPinType, state::AppState};
use axum::{
    extract::{Path, State}, response::IntoResponse,
    routing::post,
    Json,
    Router,
};
use reqwest::StatusCode;
use serde::Deserialize;

async fn create_pin_type(
    State(state): State<Arc<AppState>>,
    Json(input): Json<DTOCreatePinType>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pin_type: DBPinType = match state
        .pin_type_repo
        .create_pin_type(
            &mut tx,
            state.snowflake.lock().await.next_id().await.id,
            &input.name,
            &input.icon,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pin_type)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
    Json(input): Json<DTOCreatePin>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pin = match state
        .pin_repo
        .create_pin(
            &mut tx,
            state.snowflake.lock().await.next_id().await.id,
            &input.name,
            id,
            input.lat,
            input.long,
            &input.address,
            input.is_sponsored,
            &input.terms,
            input.opening,
            input.closing,
            &input.instruction,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pin)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_pin_type))
        .route("/{id}", post(create_pin))
}

#[derive(Debug, Deserialize)]
pub struct DTOCreatePinType {
    name: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
pub struct DTOCreatePin {
    name: String,
    lat: f32,
    long: f32,
    address: String,
    is_sponsored: bool,
    terms: String,
    opening: Vec<i32>,
    closing: Vec<i32>,
    instruction: String,
}
