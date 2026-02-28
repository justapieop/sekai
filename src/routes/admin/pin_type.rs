use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{repo::pin_types::DBPinType, state::AppState};

async fn create_pin_type(
    State(state): State<Arc<AppState>>,
    TypedMultipart(input): TypedMultipart<DTOCreatePinType>,
) -> impl IntoResponse {
    let pin_type: DBPinType = match state
        .pin_type_repo
        .create_pin_type(
            &state.pool,
            state.snowflake.lock().await.next_id().await.id,
            &input.name,
            input.icon.to_vec(),
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };
    (StatusCode::OK, Json(pin_type)).into_response()
}

async fn create_pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
    Json(input): Json<DTOCreatePin>,
) -> impl IntoResponse {
    let pin = match state
        .pin_repo
        .create_pin(
            &state.pool,
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
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };
    (StatusCode::OK, Json(pin)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_pin_type))
        .route("/{id}", post(create_pin))
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOCreatePinType {
    name: String,
    icon: Bytes,
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
}
