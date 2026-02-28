use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use reqwest::StatusCode;
use tracing::error;
use uuid::Uuid;

use crate::{
    repo::{challenge::DBChallenge, file::DBFileMetadata, user::DBUser},
    state::AppState,
};

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

async fn enroll_challenge(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    match state
        .challenge_repo
        .enroll_challenge(&state.pool, id, ext.id)
        .await
    {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => (StatusCode::BAD_REQUEST).into_response(),
    }
}

async fn upload_for_challenge(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    Path(id): Path<u128>,
    TypedMultipart(input): TypedMultipart<DTOChallengeUpload>,
) -> impl IntoResponse {
    match state.challenge_repo.get_challenge(&state.pool, id).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Challenge not found").into_response(),
    };

    let content_type: &str =
        file_type::FileType::from_bytes(&input.attachment.contents).media_types()[0];
    let file_id: u128 = state.snowflake.lock().await.next_id().await.id;
    let file_name: String = Uuid::now_v7().to_string();

    match state
        .storage_utils
        .upload_file(ext.id, input.attachment.contents, &file_name, content_type)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let metadata: DBFileMetadata = match state
        .file_repo
        .lock()
        .await
        .create_file(&state.pool, file_id, &file_name)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    match state
        .challenge_repo
        .upload_challenge(&state.pool, id, ext.id, file_id)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };
    (StatusCode::OK, Json(metadata)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(list_challenge))
        .route(
            "/{id}",
            get(get_challenge)
                .post(enroll_challenge)
                .put(upload_for_challenge),
        )
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOChallengeUpload {
    attachment: FieldData<Bytes>,
}
