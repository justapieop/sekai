use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().without_v07_checks()
}
