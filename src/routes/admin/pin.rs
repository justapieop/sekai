use std::sync::Arc;

use axum::Router;

use crate::{routes::admin::pin_type, state::AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .nest("/type", pin_type::routes())
}
