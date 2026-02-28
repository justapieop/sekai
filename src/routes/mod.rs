mod admin;
mod challenge;
mod pin;
mod pin_type;
mod post;
mod user;

use std::sync::Arc;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};
use tower::ServiceBuilder;

use crate::{middleware, state::AppState};

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .nest("/user", user::routes(state.clone()))
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
}
