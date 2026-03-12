mod admin;
mod challenge;
mod file;
mod pin;
mod pin_type;
mod post;
mod user;
mod webhook;

use std::sync::Arc;

use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};
use tower::ServiceBuilder;

use crate::{middleware, state::AppState};

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
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
}
