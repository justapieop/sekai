use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::utils::signature::Signature;
use crate::{
    repo::{
        challenge::ChallengeRepo, file::FileRepo, pin::PinRepo, pin_types::PinTypeRepo,
        post::PostRepo, user::UserRepo,
    },
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
    pub pin_repo: Arc<PinRepo>,
    pub pin_type_repo: Arc<PinTypeRepo>,
    pub challenge_repo: Arc<ChallengeRepo>,
    pub signature: Arc<Mutex<Signature>>,
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
        pin_repo: Arc<PinRepo>,
        pin_type_repo: Arc<PinTypeRepo>,
        challenge_repo: Arc<ChallengeRepo>,
        signature: Arc<Mutex<Signature>>,
    ) -> Self {
        Self {
            snowflake,
            jwt_utils,
            pool,
            user_repo,
            storage_utils,
            post_repo,
            file_repo,
            pin_repo,
            pin_type_repo,
            challenge_repo,
            signature,
        }
    }
}
