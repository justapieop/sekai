use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    repo::{post::DBPost, user::DBUser},
    state::AppState,
};

async fn get_all_posts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit: usize = match query.get("limit").map_or("0", |v| v).parse() {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "limit must be an unsigned integer").into_response();
        }
    };

    let page: usize = match query.get("page").map_or("0", |v| v).parse() {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "page must be an unsigned integer").into_response();
        }
    };

    if limit == 0 || page == 0 {
        return (
            StatusCode::BAD_REQUEST,
            "limit and page must be greater than 0",
        )
            .into_response();
    }

    let post_list: Vec<DBPost> = match state.post_repo.list_all_posts(&state.pool).await {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

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
    let post: DBPost = match state
        .post_repo
        .create_post(
            &state.pool,
            state.snowflake.lock().await.next_id().await.id,
            ext.id,
            input.content,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    for field in &input.attachments {
        if input.attachments.len() > 10 {
            break;
        }
        let content_type: &str = file_type::FileType::from_bytes(&field.contents).media_types()[0];
        let file_id: u128 = state.snowflake.lock().await.next_id().await.id;
        let file_name: String = Uuid::now_v7().to_string();

        match state
            .file_repo
            .lock()
            .await
            .create_file(&state.pool, file_id, &file_name)
            .await
        {
            Ok(s) => {
                match state
                    .storage_utils
                    .upload_file(ext.id, field.contents.clone(), &s.file_name, content_type)
                    .await
                {
                    Ok(s) => s,
                    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                };
            }
            Err(_) => {}
        }
    }

    (StatusCode::OK, Json(post)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_posts).post(create_post))
        .route("/{id}", get(get_post_by_id))
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
    #[form_data(limit = "2GiB")]
    attachments: Vec<FieldData<Bytes>>,
}
