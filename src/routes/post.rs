use crate::{
    repo::{post::DBPost, user::DBUser},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State}, response::IntoResponse, routing::get,
    Extension,
    Json,
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bigdecimal::BigDecimal;
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

async fn get_all_posts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
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

    if limit == 0 || page == 0 {
        return (
            StatusCode::BAD_REQUEST,
            "limit and page must be greater than 0",
        )
            .into_response();
    }

    let post_list: Vec<DBPost> = match state.post_repo.list_all_posts(&state.pool).await {
        Some(s) => s,
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if post_list.is_empty() {
        return (StatusCode::OK, Json(post_list)).into_response();
    }

    let chunked_post_list: Vec<&[DBPost]> = post_list.chunks(page).collect();

    (
        StatusCode::OK,
        Json(GetAllPostResponse {
            page,
            limit,
            posts: chunked_post_list[page - 1].to_vec(),
        }),
    )
        .into_response()
}

async fn get_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    let post: DBPost = match state.post_repo.get_post_by_id(&state.pool, id).await {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, "Post not found").into_response();
        }
    };

    (StatusCode::OK, Json(post)).into_response()
}

async fn create_post(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
    TypedMultipart(input): TypedMultipart<CreatePostDTO>,
) -> impl IntoResponse {
    let post_id: u128 = state.snowflake.lock().await.next_id().await.id;

    let post: DBPost = match state
        .post_repo
        .create_post(&state.pool, post_id, ext.id, &input.content)
        .await
    {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if input.attachments.is_empty() {
        return (StatusCode::OK, Json(post)).into_response();
    }

    for field in input.attachments {
        let content_type: &str = file_type::FileType::from_bytes(&field.contents).media_types()[0];
        let file_id: u128 = state.snowflake.lock().await.next_id().await.id;

        match state
            .file_repo
            .lock()
            .await
            .create_file(&state.pool, file_id, ext.id)
            .await
        {
            Ok(_) => {
                match state
                    .storage_utils
                    .upload_file(
                        ext.id,
                        field.contents.clone(),
                        &file_id.to_string(),
                        content_type,
                    )
                    .await
                {
                    Ok(s) => s,
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };
            }
            Err(_) => {}
        }

        let _ = state
            .post_repo
            .link_post_attachment(&state.pool, post_id, file_id)
            .await;
    }

    (StatusCode::OK, Json(post)).into_response()
}

async fn get_post_attachments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    match state.post_repo.get_post_attachments(&state.pool, id).await {
        None => (StatusCode::OK, Json(Vec::<BigDecimal>::new())).into_response(),
        Some(s) => (StatusCode::OK, Json(s)).into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_posts).post(create_post))
        .route("/{id}", get(get_post_by_id))
        .route("/{id}/attachment", get(get_post_attachments))
}

#[derive(Debug, Serialize)]
pub struct GetAllPostResponse {
    pub page: usize,
    pub limit: usize,
    pub posts: Vec<DBPost>,
}

#[derive(Debug, TryFromMultipart)]
pub struct CreatePostDTO {
    content: String,
    attachments: Vec<FieldData<Bytes>>,
}
