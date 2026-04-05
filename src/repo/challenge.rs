use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, Transaction};
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
    pub cover_image: BigDecimal,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBUserChallenge {
    pub user_id: Uuid,
    pub challenge_id: BigDecimal,
    pub joined_at: DateTime<Utc>,
    pub finished: bool,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBUserChallengeUploads {
    pub user_id: Uuid,
    pub challenge_id: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub attachment_id: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBDeletedAttachmentList {
    pub id: BigDecimal,
}

pub struct ChallengeRepo {
    cache: Cache<String, Vec<DBChallenge>>,
    upload_cache: Cache<Uuid, Vec<DBUserChallengeUploads>>,
}

impl ChallengeRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
            upload_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_hours(1))
                .build(),
        }
    }

    pub async fn list_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<DBChallenge>, Box<dyn Error>> {
        if let Some(cached_challenge_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_challenge_list);
        }

        let challenges: &Vec<DBChallenge> = &(match sqlx::query_as!(
            DBChallenge,
            r#"SELECT * FROM challenges WHERE CURRENT_TIMESTAMP < (ends_at + INTERVAL '3 days') ORDER BY created_at;"#
        )
        .fetch_all(&mut **pool)
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

    pub async fn get_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
    ) -> Option<DBChallenge> {
        if let Some(cached_challenge_list) = self.cache.get(CACHE_KEY).await {
            if let Some(challenge) = cached_challenge_list
                .iter()
                .find(|c| c.id == BigDecimal::from_u128(id).unwrap_or_default())
            {
                return Some(challenge.clone());
            }
        }

        sqlx::query_as!(
            DBChallenge,
            r#"SELECT * FROM challenges WHERE id = $1;"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(&mut **pool)
        .await
        .unwrap_or_else(|_| None)
    }

    pub async fn enroll_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"INSERT INTO user_challenges (user_id, challenge_id) SELECT $1, $2 WHERE NOT EXISTS (SELECT 1 FROM user_challenges uc JOIN challenges c ON c.id = uc.challenge_id WHERE uc.user_id = $1 AND c.ends_at > NOW() AND uc.finished = false) AND EXISTS (SELECT 1 FROM challenges WHERE id = $2 AND starts_at <= NOW() AND NOW() <= ends_at) ON CONFLICT DO NOTHING RETURNING *;"#,
            user_id,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        Ok(())
    }

    pub async fn delete_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"DELETE FROM challenges WHERE id = $1"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(&mut **pool)
        .await
        {
            Ok(_) => {
                self.cache.invalidate_all();
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn create_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        title: &str,
        description: &str,
        instruction: &str,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        points: i32,
        duration: i32,
        user_id: Uuid,
        cover_id: u128,
    ) -> Result<DBChallenge, Box<dyn Error>> {
        let challenge: &DBChallenge = &(match sqlx::query_as!(
                    DBChallenge,
                    r#"INSERT INTO challenges (id, title, description, instruction, ends_at, points, duration, starts_at, created_by, cover_image) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *;"#,
                    BigDecimal::from_u128(id).unwrap_or_default(),
                    title,
                    description,
                    instruction,
                    ends_at,
                    points,
                    duration,
                    starts_at,
                    user_id,
                    BigDecimal::from_u128(cover_id).unwrap_or_default(),
                ).fetch_one(&mut **pool).await {
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
        pool: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> Option<DBUserChallenge> {
        match sqlx::query_as!(
            DBUserChallenge,
            r#"SELECT * FROM user_challenges uc WHERE user_id = $1 AND (SELECT ends_at FROM challenges WHERE id = uc.challenge_id) > CURRENT_TIMESTAMP ORDER BY joined_at DESC LIMIT 1;"#,
            user_id
        )
        .fetch_one(&mut **pool)
        .await
        {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub async fn upload_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        user_id: Uuid,
        file_id: u128,
    ) -> Result<DBUserChallengeUploads, Box<dyn Error>> {
        let upload: &DBUserChallengeUploads = &(match sqlx::query_as!(DBUserChallengeUploads, r#"INSERT INTO user_challenge_uploads (user_id, challenge_id, attachment_id) VALUES($1, $2, $3) RETURNING *;"#,
            user_id,
            BigDecimal::from_u128(id).unwrap_or_default(),
            BigDecimal::from_u128(file_id).unwrap_or_default()
        ).fetch_one(&mut **pool).await {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        if let Some(mut cached_upload_list) = self.upload_cache.get(&user_id).await {
            cached_upload_list.push(upload.clone());
            self.upload_cache.insert(user_id, cached_upload_list).await;
        }

        Ok(upload.clone())
    }

    pub async fn get_user_uploads(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        challenge_id: u128,
    ) -> Option<Vec<DBUserChallengeUploads>> {
        if let Some(cached_upload_list) = self.upload_cache.get(&user_id).await {
            return Some(cached_upload_list);
        }

        let uploads_list: &Vec<DBUserChallengeUploads> = &(match sqlx::query_as!(
            DBUserChallengeUploads,
            r#"SELECT * FROM user_challenge_uploads WHERE user_id = $1 AND challenge_id = $2 ORDER BY created_at;"#,
            user_id,
            BigDecimal::from_u128(challenge_id).unwrap_or_default(),
        )
        .fetch_all(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        });

        self.upload_cache
            .insert(user_id, uploads_list.clone())
            .await;

        Some(uploads_list.clone())
    }

    pub async fn finish_challenge(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        challenge_id: u128,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            r#"UPDATE user_challenges SET finished = true, finished_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND challenge_id = $2;"#,
            user_id,
            BigDecimal::from_u128(challenge_id).unwrap_or_default(),
        ).fetch_optional(&mut **pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }
    pub async fn withdraw(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        challenge_id: u128,
    ) -> Result<Vec<DBDeletedAttachmentList>, Box<dyn Error>> {
        match sqlx::query_as!(
            DBDeletedAttachmentList,
            r#"WITH deleted_uploads AS (DELETE FROM user_challenge_uploads WHERE user_id = $1 AND challenge_id = $2 RETURNING attachment_id), deleted_challenge AS (DELETE FROM user_challenges WHERE user_id = $1 AND challenge_id = $2) DELETE FROM file_metadata WHERE id IN (SELECT attachment_id FROM deleted_uploads) RETURNING id;"#,
            user_id,
            BigDecimal::from_u128(challenge_id).unwrap_or_default()
        )
        .fetch_all(&mut **pool)
        .await
        {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into()),
        }
    }
}
