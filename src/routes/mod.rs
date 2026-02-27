use std::sync::Arc;

use axum::Router;

use crate::state::AppState;

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new().without_v07_checks()
}
