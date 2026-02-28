mod pin;
mod pin_type;

use std::sync::Arc;

use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .without_v07_checks()
        .nest("/pin", pin::routes())
}
