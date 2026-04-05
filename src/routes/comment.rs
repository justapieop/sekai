use crate::repo::comment::DBComment;
use crate::repo::user::DBUser;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use std::sync::Arc;

async fn get_comment_from_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let (limit, page): (usize, usize) = (
        if let Ok(s) = query.get("limit").map_or("0", |v| v).parse() {
            s
        } else {
            return (StatusCode::BAD_REQUEST, "limit must be an unsigned integer").into_response();
        },
        if let Ok(s) = query.get("page").map_or("0", |v| v).parse() {
            s
        } else {
            return (StatusCode::BAD_REQUEST, "page must be an unsigned integer").into_response();
        },
    );

    let comments: Vec<DBComment> = state
        .comment_repo
        .get_post_comments(&mut tx, id)
        .await
        .unwrap();

    let chunked: Vec<&[DBComment]> = comments.chunks(limit).collect();

    match tx.commit().await {
        Ok(_) => (
            StatusCode::OK,
            Json(GetAllCommentResponse {
                page,
                limit,
                comments: Vec::from(chunked[page - 1]),
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_replies(
    State(state): State<Arc<AppState>>,
    Path((_, comment_id)): Path<(u128, u128)>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let (limit, page): (usize, usize) = (
        if let Ok(s) = query.get("limit").map_or("0", |v| v).parse() {
            s
        } else {
            return (StatusCode::BAD_REQUEST, "limit must be an unsigned integer").into_response();
        },
        if let Ok(s) = query.get("page").map_or("0", |v| v).parse() {
            s
        } else {
            return (StatusCode::BAD_REQUEST, "page must be an unsigned integer").into_response();
        },
    );

    let comments: Vec<DBComment> = state
        .comment_repo
        .get_replies(&mut tx, comment_id)
        .await
        .unwrap();

    let chunked: Vec<&[DBComment]> = comments.chunks(limit).collect();

    match tx.commit().await {
        Ok(_) => (
            StatusCode::OK,
            Json(GetAllCommentResponse {
                page,
                limit,
                comments: Vec::from(chunked[page - 1]),
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn post_comment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
    Extension(ext): Extension<Arc<DBUser>>,
    TypedMultipart(input): TypedMultipart<DTOCreateComment>,
) -> impl IntoResponse {
    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let comment_id: u128 = state.snowflake.lock().await.next_id().await.id;

    let file_id: Option<u128> = if input.attachment.is_some() {
        let id: u128 = state.snowflake.lock().await.next_id().await.id;

        let content: FieldData<Bytes> = input.attachment.unwrap();

        let content_type: &str =
            file_type::FileType::from_bytes(&content.contents).media_types()[0];

        match state
            .file_repo
            .lock()
            .await
            .create_file(&mut tx, id, ext.id)
            .await
        {
            Ok(_) => match state
                .storage_utils
                .upload_file(ext.id, content.contents, &id.to_string(), content_type)
                .await
            {
                Ok(s) => s,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        Some(id)
    } else {
        None
    };

    let comment: DBComment = match state
        .comment_repo
        .post_comment(
            &mut tx,
            comment_id,
            id,
            ext.id,
            &input.content,
            file_id,
            input.reply_to,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(comment)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_comment_from_post).post(post_comment))
        .route("/{id}", get(get_replies))
}

#[derive(Debug, Serialize)]
pub struct GetAllCommentResponse {
    page: usize,
    limit: usize,
    comments: Vec<DBComment>,
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOCreateComment {
    content: String,
    attachment: Option<FieldData<Bytes>>,
    reply_to: Option<u128>,
}
