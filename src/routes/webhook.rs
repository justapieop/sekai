use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(input): Json<WebhookRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    match state
        .user_repo
        .create_profile(
            &mut tx,
            input.payload.user.id,
            &input.payload.user.standard_attributes.email,
            &input.payload.user.standard_attributes.name,
            &input.payload.user.standard_attributes.picture,
        )
        .await
    {
        Ok(_) => {}
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match tx.commit().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_user))
}

#[derive(Deserialize)]
pub struct WebhookRequest {
    pub r#type: String,
    pub payload: Payload,
}

#[derive(Deserialize)]
pub struct Payload {
    pub user: User,
}

#[derive(Deserialize)]
pub struct User {
    pub standard_attributes: StandardAttributes,
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct StandardAttributes {
    pub email: String,
    pub name: String,
    pub picture: String,
}
