use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

const CACHE_KEY: &str = "CHALLENGE_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBChallenge {
    pub id: BigDecimal,
    pub title: String,
    pub description: String,
    pub instruction: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub points: i32,
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

        let challenges: &Vec<DBChallenge> =
            &(match sqlx::query_as!(DBChallenge, r#"SELECT * FROM challenges;"#)
                .fetch_all(pool)
                .await
            {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            });

        self.cache
            .insert(String::from(CACHE_KEY), challenges.clone())
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
}
