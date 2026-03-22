use std::sync::Arc;

use crate::repo::user::DBUser;
use crate::{repo::pin_types::DBPinType, state::AppState};
use axum::{
    extract::{Path, State}, response::IntoResponse, routing::post,
    Extension,
    Json,
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{Error, Postgres, Transaction};
use tracing::error;

async fn create_pin_type(
    State(state): State<Arc<AppState>>,
    Json(input): Json<DTOCreatePinType>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let pin_type: DBPinType = match state
        .pin_type_repo
        .create_pin_type(
            &mut tx,
            state.snowflake.lock().await.next_id().await.id,
            &input.name,
            &input.icon,
        )
        .await
    {
        Ok(s) => s,
        Err(e) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pin_type)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
    Extension(ext): Extension<Arc<DBUser>>,
    TypedMultipart(input): TypedMultipart<DTOCreatePin>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let file_id: u128 = state.snowflake.lock().await.next_id().await.id;
    let content_type: &str =
        file_type::FileType::from_bytes(&input.attachment.contents).media_types()[0];

    match state
        .file_repo
        .lock()
        .await
        .create_file(&mut tx, file_id, ext.id)
        .await
    {
        Ok(_) => match state
            .storage_utils
            .upload_public_file(
                &input.attachment.contents,
                &file_id.to_string(),
                content_type,
            )
            .await
        {
            Ok(_) => {}
            Err(e) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let pin = match state
        .pin_repo
        .create_pin(
            &mut tx,
            state.snowflake.lock().await.next_id().await.id,
            &input.name,
            id,
            input.lat,
            input.long,
            &input.address,
            input.is_sponsored,
            &input.terms,
            input.opening,
            input.closing,
            &input.instruction,
            file_id,
            &input.accepts,
            input.opening_days,
            &input.note,
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match tx.commit().await {
        Ok(_) => (StatusCode::OK, Json(pin)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_routes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u128>,
) -> impl IntoResponse {
    let mut tx: Transaction<Postgres> = match state.pool.begin().await {
        Ok(s) => s,
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match state.pin_type_repo.delete_pin_type(&mut tx, id).await {
        Ok(_) => {}
        Err(e) => {
            error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match tx.commit().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", post(create_pin_type))
        .route("/{id}", post(create_pin).delete(delete_routes))
}

#[derive(Debug, Deserialize)]
pub struct DTOCreatePinType {
    name: String,
    icon: String,
}

#[derive(Debug, TryFromMultipart)]
pub struct DTOCreatePin {
    name: String,
    lat: f32,
    long: f32,
    address: String,
    is_sponsored: bool,
    terms: String,
    opening: Vec<i32>,
    closing: Vec<i32>,
    instruction: String,
    attachment: FieldData<Bytes>,
    accepts: String,
    opening_days: i16,
    note: String,
}
