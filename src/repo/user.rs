use std::{error::Error, time::Duration};

use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBUser {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bio: String,
    pub is_admin: bool,
    pub points: i128,
    pub email: String,
    pub name: String,
    pub avatar_url: String,
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

    pub async fn get_all_user(
        &self,
        pool: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<DBUser>, Box<dyn Error>> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_user_list);
        }
        let user_list: &Vec<DBUser> = &(match sqlx::query_as!(DBUser, r#"SELECT * FROM users;"#)
            .fetch_all(&mut **pool)
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

    pub async fn get_user_by_id(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Option<DBUser> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            let user: Option<DBUser> = match cached_user_list.iter().find(|u| u.id == id) {
                Some(s) => Some(s.to_owned()),
                None => None,
            };

            if user.is_some() {
                return Some(user.unwrap());
            }
        }

        sqlx::query_as!(DBUser, r#"SELECT * FROM users WHERE id = $1"#, id)
            .fetch_optional(&mut **pool)
            .await
            .unwrap_or_else(|_| None)
    }

    pub async fn create_profile(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: Uuid,
        email: &str,
        name: &str,
        avatar_url: &str,
    ) -> Result<DBUser, Box<dyn Error>> {
        if let Some(cached_user_list) = self.cache.get(CACHE_KEY).await {
            if let Some(user) = cached_user_list.iter().find(|u| u.id == id) {
                return Ok(user.clone());
            }
        }

        let user: &DBUser = &(match sqlx::query_as!(
            DBUser,
            r#"INSERT INTO users(id, email, name, avatar_url) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, name = EXCLUDED.name, avatar_url = excluded.avatar_url RETURNING *;"#,
            id,
            email,
            name,
            avatar_url
        )
        .fetch_one(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        });

        if let Some(mut cached_user_list) = self.cache.get(CACHE_KEY).await {
            cached_user_list.push(user.clone());
            self.cache
                .insert(String::from(CACHE_KEY), cached_user_list)
                .await;
        }

        Ok(user.clone())
    }

    pub async fn update_bio(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        bio: &str,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!("UPDATE users SET bio = $1 WHERE id = $2;", bio, user_id)
            .fetch_optional(&mut **pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
