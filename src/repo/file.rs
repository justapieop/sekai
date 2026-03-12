use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBFileMetadata {
    pub id: BigDecimal,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

pub struct FileRepo {
    cache: Cache<u128, DBFileMetadata>,
}

impl FileRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_hours(3))
                .build(),
        }
    }

    pub async fn get_file_by_id(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
    ) -> Option<DBFileMetadata> {
        if let Some(file) = self.cache.get(&id).await {
            return Some(file);
        }

        let file: &DBFileMetadata = &(match sqlx::query_as!(
            DBFileMetadata,
            r#"SELECT * FROM file_metadata WHERE id = $1;"#,
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_one(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        });

        self.cache.insert(id.clone(), file.clone()).await;

        Some(file.clone())
    }

    pub async fn create_file(
        &mut self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        user_id: Uuid,
    ) -> Result<DBFileMetadata, Box<dyn Error + Send + Sync>> {
        let file: &DBFileMetadata = &(match sqlx::query_as!(
            DBFileMetadata,
            r#"INSERT INTO file_metadata(id, uploaded_by) VALUES($1, $2) RETURNING *;"#,
            BigDecimal::from_u128(id).unwrap_or_default(),
            user_id
        )
        .fetch_one(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        if self.cache.get(&id).await.is_none() {
            self.cache.insert(id, file.clone()).await;
        }

        Ok(file.clone())
    }
}
