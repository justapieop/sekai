use std::{str::FromStr, sync::Arc};

use crate::{AppState, repo::user::DBUser};

use tracing::{debug, error};

use axum::{
    Extension,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
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
            .create_profile(
                &state.pool,
                match Uuid::from_str(&uid) {
                    Ok(s) => s,
                    Err(_) => {
                        return res;
                    }
                },
            )
            .await
        {
            Ok(s) => s,
            Err(e) => {
                debug!("{}", e);
                return res;
            }
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
