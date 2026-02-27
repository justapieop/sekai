use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::utils::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator};

#[derive(Clone)]
pub struct AppState {
    pub snowflake: Arc<Mutex<SnowflakeGenerator>>,
    pub jwt_utils: Arc<JwtUtils>,
    pub pool: PgPool,
}

impl AppState {
    pub fn new(
        snowflake: Arc<Mutex<SnowflakeGenerator>>,
        jwt_utils: Arc<JwtUtils>,
        pool: PgPool,
    ) -> Self {
        Self {
            snowflake,
            jwt_utils,
            pool,
        }
    }
}
