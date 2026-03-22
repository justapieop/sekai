mod admin;
mod ai;
mod challenge;
mod comment;
mod file;
mod pin;
mod pin_type;
mod post;
mod user;
mod webhook;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};
use tower::ServiceBuilder;

use crate::{middleware, state::AppState};

async fn home() -> impl IntoResponse {
    (StatusCode::OK).into_response()
}

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(home))
        .nest(
            "/user",
            user::routes().layer(from_fn_with_state(
                state.clone(),
                middleware::verify_access_token,
            )),
        )
        .nest(
            "/post",
            post::routes().layer(from_fn_with_state(
                state.clone(),
                middleware::verify_access_token,
            )),
        )
        .nest("/pin", pin::routes())
        .nest(
            "/admin",
            admin::routes().layer(
                ServiceBuilder::new()
                    .layer(from_fn_with_state(
                        state.clone(),
                        middleware::verify_access_token,
                    ))
                    .layer(from_fn(middleware::restrict_admin)),
            ),
        )
        .nest(
            "/challenge",
            challenge::routes().layer(from_fn_with_state(
                state.clone(),
                middleware::verify_access_token,
            )),
        )
        .nest("/file", file::routes())
        .nest(
            "/webhook",
            webhook::routes().layer(from_fn_with_state(
                state.clone(),
                middleware::check_signature,
            )),
        )
        .nest(
            "/ai",
            ai::routes()
                .layer(from_fn_with_state(
                    state.clone(),
                    middleware::verify_access_token,
                ))
                .layer(from_fn_with_state(
                    state.clone(),
                    middleware::check_balance(1),
                )),
        )
}
