use std::sync::Arc;

use crate::{repo::user::DBUser, state::AppState};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, post},
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;

async fn create_challenge(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    TypedMultipart(input): TypedMultipart<DTOCreateChallenge>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let file_id: u128 = { state.snowflake.lock().await.next_id().await.id };

    match state
        .file_repo
        .lock()
        .await
        .create_file(&mut tx, file_id, ext.id)
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let content_type: &str =
        file_type::FileType::from_bytes(&input.cover_image.contents).media_types()[0];

    match state
        .storage_utils
        .upload_public_file(
            &input.cover_image.contents,
            &file_id.to_string(),
            content_type,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let challenge_id: u128 = { state.snowflake.lock().await.next_id().await.id };

    let result = match state
        .challenge_repo
        .create_challenge(
            &mut tx,
            challenge_id,
            &input.title,
            &input.description,
            &input.instruction,
            input.starts_at,
            input.ends_at,
            input.points,
            input.duration,
            ext.id,
            file_id,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(result)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_challenge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    match state.challenge_repo.delete_challenge(&mut tx, id).await {
        Ok(_) => {}
        Err(_) => {}
    };

    match tx.commit().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_challenge))
        .route("/{id}", delete(delete_challenge))
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOCreateChallenge {
    title: String,
    description: String,
    instruction: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    points: i32,
    duration: i32,
    cover_image: FieldData<Bytes>,
}
