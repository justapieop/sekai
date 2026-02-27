use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{
    repo::user::UserRepo,
    utils::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator},
};

#[derive(Clone)]
pub struct AppState {
    pub snowflake: Arc<Mutex<SnowflakeGenerator>>,
    pub jwt_utils: Arc<JwtUtils>,
    pub pool: PgPool,
    pub user_repo: Arc<UserRepo>,
}

impl AppState {
    pub fn new(
        snowflake: Arc<Mutex<SnowflakeGenerator>>,
        jwt_utils: Arc<JwtUtils>,
        pool: PgPool,
        user_repo: Arc<UserRepo>,
    ) -> Self {
        Self {
            snowflake,
            jwt_utils,
            pool,
            user_repo,
        }
    }
}
