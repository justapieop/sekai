use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{
    repo::{file::FileRepo, post::PostRepo, user::UserRepo},
    utils::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator, storage::StorageUtils},
};

#[derive(Clone)]
pub struct AppState {
    pub snowflake: Arc<Mutex<SnowflakeGenerator>>,
    pub jwt_utils: Arc<JwtUtils>,
    pub pool: PgPool,
    pub storage_utils: Arc<StorageUtils>,
    pub user_repo: Arc<UserRepo>,
    pub post_repo: Arc<PostRepo>,
    pub file_repo: Arc<Mutex<FileRepo>>,
}

impl AppState {
    pub fn new(
        snowflake: Arc<Mutex<SnowflakeGenerator>>,
        jwt_utils: Arc<JwtUtils>,
        pool: PgPool,
        storage_utils: Arc<StorageUtils>,
        user_repo: Arc<UserRepo>,
        post_repo: Arc<PostRepo>,
        file_repo: Arc<Mutex<FileRepo>>,
    ) -> Self {
        Self {
            snowflake,
            jwt_utils,
            pool,
            user_repo,
            storage_utils,
            post_repo,
            file_repo,
        }
    }
}
