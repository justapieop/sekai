use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{jwt_utils::JwtUtils, snowflake::SnowflakeGenerator};

pub struct AppState {
    pub snowflake: Arc<Mutex<SnowflakeGenerator>>,
    pub jwt_utils: Arc<JwtUtils>,
}

impl AppState {
    pub fn new(snowflake: Arc<Mutex<SnowflakeGenerator>>, jwt_utils: Arc<JwtUtils>) -> Self {
        Self {
            snowflake,
            jwt_utils,
        }
    }
}
