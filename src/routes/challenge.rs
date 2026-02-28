use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;

use crate::{repo::challenge::DBChallenge, state::AppState};

async fn list_challenge(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let challenges: Vec<DBChallenge> = match state.challenge_repo.list_challenge(&state.pool).await
    {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    (StatusCode::OK, Json(challenges)).into_response()
}

async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    let challenge: DBChallenge = match state.challenge_repo.get_challenge(&state.pool, id).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Challenge not found").into_response(),
    };

    (StatusCode::OK, Json(challenge)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(list_challenge))
        .route("/{id}", get(get_challenge))
}
