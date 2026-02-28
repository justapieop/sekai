pub mod post;
pub mod user;

use std::sync::Arc;

use axum::{Router, middleware::from_fn_with_state};

use crate::{middleware, state::AppState};

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .nest("/user", user::routes())
        .nest(
            "/post",
            post::routes().layer(from_fn_with_state(
                state.clone(),
                middleware::verify_access_token,
            )),
        )
}
