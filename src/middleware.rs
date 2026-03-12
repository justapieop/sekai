use std::{str::FromStr, sync::Arc};

use crate::{repo::user::DBUser, AppState};

use tracing::error;

use axum::body::to_bytes;
use axum::http::HeaderValue;
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
        Err(_) => {
            error!("Invalid Authorization header");
            return res;
        }
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

    let user: Arc<DBUser> = Arc::new(
        match state
            .user_repo
            .get_user_by_id(&state.pool, Uuid::from_str(&uid).unwrap_or_default())
            .await
        {
            None => return res,
            Some(s) => s,
        },
    );

    req.extensions_mut().insert(user);

    next.run(req).await
}

pub async fn restrict_admin(
    Extension(ext): Extension<Arc<DBUser>>,
    req: Request,
    next: Next,
) -> Response {
    if !ext.is_admin {
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
    let signature: &HeaderValue = match parts.headers.get("x-authgear-body-signature") {
        None => return StatusCode::BAD_REQUEST.into_response(),
        Some(s) => s,
    };
    let body_bytes: Bytes = match to_bytes(body, usize::MAX).await {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if !state
        .signature
        .lock()
        .await
        .verify(body_bytes.clone().iter().as_slice(), signature.as_bytes())
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let new_body: Body = Body::from(body_bytes.clone());

    let new_req: Request = Request::from_parts(parts, new_body);

    next.run(new_req).await
}
