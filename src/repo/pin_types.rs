use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, Transaction};

const CACHE_KEY: &str = "PIN_TYPE_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBPinType {
    pub id: BigDecimal,
    pub name: String,
    pub icon: Vec<u8>,
}

pub struct PinTypeRepo {
    cache: Cache<String, Vec<DBPinType>>,
}

impl PinTypeRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(3))
                .build(),
        }
    }

    pub async fn get_all_pin_types(
        &self,
        pool: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<DBPinType>, Box<dyn Error>> {
        if let Some(cached_pin_types) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_pin_types);
        }

        let pin_types: &Vec<DBPinType> =
            &(match sqlx::query_as!(DBPinType, r#"SELECT * FROM pin_types;"#)
                .fetch_all(&mut **pool)
                .await
            {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            });

        self.cache
            .insert(String::from(CACHE_KEY), pin_types.clone())
            .await;

        Ok(pin_types.clone())
    }

    pub async fn create_pin_type(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        name: &str,
        icon: Vec<u8>,
    ) -> Result<DBPinType, Box<dyn Error>> {
        let pin_type: DBPinType = match sqlx::query_as!(
            DBPinType,
            r#"INSERT INTO pin_types VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING *;"#,
            BigDecimal::from_u128(id).unwrap_or_default(),
            name,
            &icon
        )
        .fetch_one(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        Ok(pin_type)
    }

    pub async fn delete_pin_type(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
    ) -> Result<(), Box<dyn Error>> {
        match sqlx::query!(
            "DELETE FROM pin_types WHERE id = $1;",
            BigDecimal::from_u128(id).unwrap_or_default()
        )
        .fetch_optional(&mut **pool)
        .await
        {
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        };
        Ok(())
    }
}
