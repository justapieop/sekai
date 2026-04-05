use crate::{repo::user::DBUser, AppState};
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};
use bytes::Bytes;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

pub async fn verify_access_token(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();

    let auth_header = match headers.get("Authorization") {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Missing Authorization header"))
                .unwrap_or_default();
        }
    };

    let res = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid Authorization header"))
        .unwrap_or_default();

    let auth_header_value_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return res,
    };

    if auth_header_value_str.is_empty() || !auth_header_value_str.starts_with("Bearer ") {
        return res;
    }

    let tokens: Vec<&str> = auth_header_value_str.split(" ").collect();

    if tokens.len() != 2 {
        return res;
    }

    let jwt: &str = tokens[1];

    let uid: String = match state.jwt_utils.verify(jwt) {
        Ok(s) => s,
        Err(_) => {
            return res;
        }
    };

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let user: Arc<DBUser> = Arc::new(
        match state
            .user_repo
            .get_user_by_id(&mut tx, Uuid::from_str(&uid).unwrap_or_default())
            .await
        {
            None => return res,
            Some(s) => s,
        },
    );

    match tx.commit().await {
        Ok(_) => req.extensions_mut().insert(user),
        Err(_) => return res,
    };

    next.run(req).await
}

pub async fn restrict_admin(
    Extension(ext): Extension<Arc<DBUser>>,
    req: Request,
    next: Next,
) -> Response {
    if !ext.admin {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Unauthorized"))
            .unwrap_or_default();
    }
    next.run(req).await
}

pub async fn check_signature(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let (parts, body) = req.into_parts();

    let signature_header = match parts.headers.get("x-authgear-body-signature") {
        None => return StatusCode::BAD_REQUEST.into_response(),
        Some(s) => s,
    };

    let signature_hex = match signature_header.to_str() {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let received_hmac_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let body_bytes: Bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if !state.signature.verify(&body_bytes, &received_hmac_bytes) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let new_body = Body::from(body_bytes);
    let new_req = Request::from_parts(parts, new_body);

    let response = next.run(new_req).await;

    response
}
