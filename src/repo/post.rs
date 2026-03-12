use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

const CACHE_KEY: &str = "POST_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBPost {
    pub id: BigDecimal,
    pub author: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub likes: BigDecimal,
}

pub struct PostRepo {
    cache: Cache<String, Vec<DBPost>>,
}

impl PostRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
        }
    }

    pub async fn link_post_attachment(
        &self,
        pool: &PgPool,
        post_id: u128,
        attachment_id: u128,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"INSERT INTO post_attachments (post_id, attachment_id) VALUES ($1, $2)"#,
            BigDecimal::from_u128(post_id).unwrap_or_default(),
            BigDecimal::from_u128(attachment_id).unwrap_or_default()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_post_attachments(&self, pool: &PgPool, id: u128) -> Option<Vec<BigDecimal>> {
        match sqlx::query_scalar!(
            r#"SELECT attachment_id FROM post_attachments WHERE post_id = $1;"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_all(pool)
        .await
        {
            Ok(s) => Some(s),
            Err(_) => return None,
        }
    }

    pub async fn list_all_posts(&self, pool: &PgPool) -> Option<Vec<DBPost>> {
        if let Some(cached_post_list) = self.cache.get(CACHE_KEY).await {
            return Some(cached_post_list);
        }

        let post_list: &Vec<DBPost> =
            &(match sqlx::query_as!(DBPost, r#"SELECT * FROM posts ORDER BY updated_at;"#)
                .fetch_all(pool)
                .await
            {
                Ok(s) => s,
                Err(e) => return None,
            });

        self.cache
            .insert(String::from(CACHE_KEY), post_list.clone())
            .await;

        Some(post_list.clone())
    }

    pub async fn get_post_by_id(&self, pool: &PgPool, id: u128) -> Option<DBPost> {
        if let Some(cached_post_list) = self.cache.get(CACHE_KEY).await {
            let post: Option<DBPost> = match cached_post_list
                .iter()
                .find(|p| p.id == BigDecimal::from_u128(id).unwrap_or_default())
            {
                Some(s) => Some(s.to_owned()),
                None => None,
            };

            if post.is_some() {
                return Some(post.unwrap());
            }
        }

        let post_list: Vec<DBPost> = match self.list_all_posts(pool).await {
            Some(s) => s,
            None => return None,
        };

        let post: &DBPost = match post_list
            .iter()
            .find(|p| p.id == BigDecimal::from_u128(id).unwrap_or_default())
        {
            Some(s) => s,
            None => return None,
        };

        Some(post.to_owned())
    }

    pub async fn create_post(
        &self,
        pool: &PgPool,
        id: u128,
        author_id: Uuid,
        content: &str,
    ) -> Result<DBPost, Box<dyn Error>> {
        let post: &DBPost = &(match sqlx::query_as!(
            DBPost,
            r#"INSERT INTO posts (id, author, content) VALUES ($1, $2, $3) RETURNING *;"#,
            BigDecimal::from_u128(id).unwrap_or_default(),
            author_id,
            content
        )
        .fetch_one(pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        if let Some(mut cached_post_list) = self.cache.get(CACHE_KEY).await {
            cached_post_list.push(post.clone());
            self.cache
                .insert(String::from(CACHE_KEY), cached_post_list)
                .await;
        }

        Ok(post.clone())
    }
}
