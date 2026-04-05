use crate::repo::user::DBUser;
use crate::state::AppState;
use crate::utils::ai::AiResponse;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

async fn prompt_with_image(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    TypedMultipart(input): TypedMultipart<DTOAiPrompt>,
) -> impl IntoResponse {
    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let res: AiResponse = match state
        .ai_utils
        .prompt_with_image(&input.prompt, input.attachment.contents)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match state
        .user_repo
        .set_point(&mut tx, ext.id, ext.points - 1)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(res)).into_response(),
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

async fn prompt(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let prompt: &str = match query.get("prompt") {
        None => return StatusCode::BAD_REQUEST.into_response(),
        Some(s) => s,
    };

    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let res: AiResponse = match state.ai_utils.prompt(prompt).await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match state
        .user_repo
        .set_point(&mut tx, ext.id, ext.points - 1)
        .await
    {
        Ok(_) => {}
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(res)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/image", post(prompt_with_image))
        .route("/text", get(prompt))
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOAiPrompt {
    prompt: String,
    attachment: FieldData<Bytes>,
}
