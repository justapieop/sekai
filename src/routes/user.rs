use crate::repo::challenge::DBUserChallenge;
use crate::{
    repo::{challenge::DBUserChallengeUploads, user::DBUser},
    state::AppState,
};
use axum::extract::Path;
use axum::{
    extract::{Query, State}, response::IntoResponse, routing::get,
    Extension,
    Json,
    Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

async fn get_all_user(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
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

    if limit == 0 || page == 0 {
        return (
            StatusCode::BAD_REQUEST,
            "limit and page must be greater than 0",
        )
            .into_response();
    }

    let user_list: Vec<DBUser> = match state.user_repo.get_all_user(&mut tx).await {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response();
        }
    };

    let chunked_user_list: Vec<&[DBUser]> = user_list.chunks(limit).collect();
    match tx.commit().await {
        Ok(_) => (
            StatusCode::OK,
            Json(GetAllUserResponse {
                page,
                limit,
                users: chunked_user_list[page - 1].to_vec(),
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_user_challenge(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let user_challenge: DBUserChallenge = match state
        .challenge_repo
        .get_user_challenge(&mut tx, ext.id)
        .await
    {
        Some(s) => s,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(user_challenge)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_user_uploads(
    State(state): State<Arc<AppState>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let current_challenge: DBUserChallenge = match state
        .challenge_repo
        .get_user_challenge(&mut tx, ext.id)
        .await
    {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Challenge not found").into_response(),
    };

    let uploads: Vec<DBUserChallengeUploads> = match state
        .challenge_repo
        .get_user_uploads(
            &mut tx,
            ext.id,
            current_challenge.challenge_id.to_u128().unwrap(),
        )
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
                        created_at: upload.created_at,
                    });
                }
            }
            Err(_) => continue,
        };
    }

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(challenge_uploads)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_user_bio(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    Extension(ext): Extension<Arc<DBUser>>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
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
        .update_bio(&mut tx, ext.id, &decoded_bio)
        .await
    {
        Ok(_) => {}
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match tx.commit().await {
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

async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let user: DBUser = match state.user_repo.get_user_by_id(&mut tx, id).await {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(s) => s,
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_me).patch(update_user_bio))
        .route("/challenge", get(get_user_challenge))
        .route("/challenge/gallery", get(get_user_uploads))
        .route("/{id}", get(get_user_by_id))
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
    created_at: DateTime<Utc>,
}
