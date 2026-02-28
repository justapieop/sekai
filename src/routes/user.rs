use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;
use serde::Serialize;
use uuid::Uuid;

use crate::{repo::user::DBUser, state::AppState};

async fn get_all_user(
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

async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.user_repo.get_user_by_id(&state.pool, id).await {
        Some(s) => (StatusCode::OK, Json(s)).into_response(),
        None => (StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(get_all_user))
        .route("/{id}", get(get_user_by_id))
}

#[derive(Debug, Serialize)]
pub struct GetAllUserResponse {
    pub page: usize,
    pub limit: usize,
    pub users: Vec<DBUser>,
}
