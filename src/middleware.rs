use std::sync::Arc;

use crate::AppState;

use tracing::error;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct UserData {
    id: String,
}

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

    req.extensions_mut().insert(UserData {
        id: match uid.parse() {
            Ok(s) => s,
            Err(_) => {
                return res;
            }
        },
    });

    let response = next.run(req).await;

    response
}
