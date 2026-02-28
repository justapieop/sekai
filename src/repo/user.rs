use std::{error::Error, time::Duration};

use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBUser {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bio: String,
    pub is_admin: bool,
    pub points: i128,
}

const CACHE_KEY: &str = "USER_CACHE";

pub struct UserRepo {
    cache: Cache<String, Vec<DBUser>>,
}

impl UserRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
        }
    }

    pub async fn get_all_user(&self, pool: &PgPool) -> Result<Vec<DBUser>, Box<dyn Error>> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_user_list);
        }
        let user_list: &Vec<DBUser> = &(match sqlx::query_as!(DBUser, r#"SELECT * FROM users;"#)
            .fetch_all(pool)
            .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        self.cache
            .insert(String::from(CACHE_KEY), user_list.clone())
            .await;

        Ok(user_list.clone())
    }

    pub async fn get_user_by_id(&self, pool: &PgPool, id: Uuid) -> Option<DBUser> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            let user: Option<DBUser> = match cached_user_list.iter().find(|u| u.id == id) {
                Some(s) => Some(s.to_owned()),
                None => None,
            };

            if user.is_some() {
                return Some(user.unwrap());
            }
        }

        match sqlx::query_as!(DBUser, r#"SELECT * FROM users WHERE id = $1"#, id)
            .fetch_optional(pool)
            .await
        {
            Ok(s) => s,
            Err(_) => None,
        }
    }

    pub async fn create_profile(&self, pool: &PgPool, id: Uuid) -> Result<DBUser, Box<dyn Error>> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            if let Some(user) = cached_user_list.iter().find(|u| u.id == id) {
                return Ok(user.clone());
            }
        }

        let user: &DBUser = &(match sqlx::query_as!(
            DBUser,
            r#"INSERT INTO users(id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET id = users.id RETURNING *;"#,
            id
        )
        .fetch_one(pool)
        .await
        {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        });

        if let Some(mut cached_user_list) = self.cache.get(CACHE_KEY).await {
            cached_user_list.push(user.clone());
        }

        Ok(user.clone())
    }
}
