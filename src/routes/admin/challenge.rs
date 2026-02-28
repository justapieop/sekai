use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, post},
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::state::AppState;

async fn create_challenge(
    State(state): State<Arc<AppState>>,
    Json(input): Json<DTOCreateChallenge>,
) -> impl IntoResponse {
    match state
        .challenge_repo
        .create_challenge(
            &state.pool,
            state.snowflake.lock().await.next_id().await.id,
            &input.title,
            &input.description,
            &input.instruction,
            input.starts_at,
            input.ends_at,
            input.points,
            input.duration,
        )
        .await
    {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn delete_challenge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    match state.challenge_repo.delete_challenge(&state.pool, id).await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_challenge))
        .route("/{id}", delete(delete_challenge))
}

#[derive(Debug, Deserialize)]
pub struct DTOCreateChallenge {
    title: String,
    description: String,
    instruction: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    points: i32,
    duration: i32,
}
