use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

const CACHE_KEY: &str = "CHALLENGE_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBChallenge {
    pub id: BigDecimal,
    pub title: String,
    pub description: String,
    pub instruction: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub points: i32,
    pub duration: i32,
}

pub struct ChallengeRepo {
    cache: Cache<String, Vec<DBChallenge>>,
}

impl ChallengeRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
        }
    }

    pub async fn list_challenge(&self, pool: &PgPool) -> Result<Vec<DBChallenge>, Box<dyn Error>> {
        if let Some(cached_challenge_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_challenge_list);
        }

        let challenges: &Vec<DBChallenge> = &(match sqlx::query_as!(
            DBChallenge,
            r#"SELECT * FROM challenges ORDER BY created_at;"#
        )
        .fetch_all(pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        self.cache
            .insert(String::from(CACHE_KEY), challenges.to_owned())
            .await;

        Ok(challenges.clone())
    }

    pub async fn get_challenge(&self, pool: &PgPool, id: u128) -> Option<DBChallenge> {
        if let Some(cached_challenge_list) = self.cache.get(CACHE_KEY).await {
            if let Some(challenge) = cached_challenge_list
                .iter()
                .find(|c| c.id == BigDecimal::from_u128(id).unwrap_or_default())
            {
                return Some(challenge.clone());
            }
        }

        match sqlx::query_as!(
            DBChallenge,
            r#"SELECT * FROM challenges WHERE id = $1;"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(s) => s,
            Err(_) => None,
        }
    }

    pub async fn enroll_challenge(
        &self,
        pool: &PgPool,
        id: u128,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"INSERT INTO user_challenges VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING *;"#,
            user_id,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        Ok(())
    }

    pub async fn delete_challenge(&self, pool: &PgPool, id: u128) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"DELETE FROM challenges WHERE id = $1"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn create_challenge(
        &self,
        pool: &PgPool,
        id: u128,
        title: &str,
        description: &str,
        instruction: &str,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        points: i32,
        duration: i32,
    ) -> Result<DBChallenge, Box<dyn Error>> {
        let challenge: &DBChallenge = &(match sqlx::query_as!(
                    DBChallenge,
                    r#"INSERT INTO challenges (id, title, description, instruction, ends_at, points, duration, starts_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *;"#,
                    BigDecimal::from_u128(id).unwrap_or_default(),
                    title,
                    description,
                    instruction,
                    ends_at,
                    points,
                    duration,
                    starts_at
                ).fetch_one(pool).await {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        if let Some(mut cached_challenge_list) = self.cache.get(CACHE_KEY).await {
            cached_challenge_list.push(challenge.to_owned());
            self.cache
                .insert(String::from(CACHE_KEY), cached_challenge_list)
                .await;
        }

        self.cache.invalidate_all();

        Ok(challenge.clone())
    }

    pub async fn get_user_challenge(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<DBChallenge, Box<dyn Error>> {
        match sqlx::query_as!(
                    DBChallenge,
                    r#"SELECT * FROM challenges WHERE id = (SELECT challenge_id FROM user_challenges WHERE user_id = $1 ORDER BY created_at DESC);"#,
                    user_id
                ).fetch_one(pool).await {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn upload_challenge(
        &self,
        pool: &PgPool,
        id: u128,
        user_id: Uuid,
        file_id: u128,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(r#"INSERT INTO user_challenges_uploads (user_id, challenge_id, attachment_id) VALUES($1, $2, $3);"#,
            user_id,
            BigDecimal::from_u128(id).unwrap_or_default(),
            BigDecimal::from_u128(file_id).unwrap_or_default()
        ).fetch_optional(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
