use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State}, response::IntoResponse, routing::get,
    Extension,
    Json,
    Router,
};
use bigdecimal::BigDecimal;
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    repo::{challenge::DBUserChallengeUploads, user::DBUser},
    state::AppState,
};

async fn get_all_user(
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

    let user_list: Vec<DBUser> = match state.user_repo.get_all_user(&state.pool).await {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response();
        }
    };

    let chunked_user_list: Vec<&[DBUser]> = user_list.chunks(limit).collect();
    (
        StatusCode::OK,
        Json(GetAllUserResponse {
            page,
            limit,
            users: chunked_user_list[page - 1].to_vec(),
        }),
    )
        .into_response()
}

async fn get_user_challenge(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    match state
        .challenge_repo
        .get_user_challenge(&state.pool, ext.id)
        .await
    {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_user_uploads(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    let uploads: Vec<DBUserChallengeUploads> = match state
        .challenge_repo
        .get_user_uploads(&state.pool, ext.id)
        .await
    {
        Some(s) => s,
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut challenge_uploads: Vec<GetAllUserUploadsResponse> = Vec::new();

    for upload in uploads {
        match state
            .storage_utils
            .fetch_file(ext.id, &upload.attachment_id.to_string())
            .await
        {
            Ok(s) => {
                if let Ok(bytes) = s.bytes().await {
                    challenge_uploads.push(GetAllUserUploadsResponse {
                        challenge_id: upload.challenge_id,
                        content: bytes,
                    });
                }
            }
            Err(_) => continue,
        };
    }

    (StatusCode::OK, Json(challenge_uploads)).into_response()
}

async fn update_user_bio(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    let bio: String = match query.get("bio_value") {
        Some(s) => s.to_owned(),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let decoded_bio = match urlencoding::decode(&bio) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match state
        .user_repo
        .update_bio(&state.pool, ext.id, &decoded_bio)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_me(
    State(_): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(ext)).into_response()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_me).patch(update_user_bio))
        .route("/challenge", get(get_user_challenge))
        .route("/gallery", get(get_user_uploads))
}

#[derive(Debug, Serialize)]
pub struct GetAllUserResponse {
    pub page: usize,
    pub limit: usize,
    pub users: Vec<DBUser>,
}

#[derive(Debug, Serialize)]
pub struct GetAllUserUploadsResponse {
    challenge_id: BigDecimal,
    content: Bytes,
}
