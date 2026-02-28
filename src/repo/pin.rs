use std::{error::Error, time::Duration};

use bigdecimal::BigDecimal;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, PgPool,
    types::{Json, JsonRawValue},
};

const CACHE_KEY: &str = "PIN_CACHE";
const PIN_TYPE_CACHE_KEY: &str = "PIN_TYPE_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBPin {
    pub id: BigDecimal,
    pub name: String,
    pub type_id: BigDecimal,
    pub lat: f64,
    pub long: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub address: String,
    pub is_sponsored: bool,
    pub terms: String,
    pub opening: Vec<i32>,
    pub closing: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PinType {
    pub id: BigDecimal,
    pub name: String,
    pub icon: Bytes,
}

pub struct PinRepo {
    cache: Cache<String, Vec<DBPin>>,
    pin_types_cache: Cache<String, Vec<PinType>>,
}

impl PinRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
            pin_types_cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(3))
                .build(),
        }
    }

    pub async fn get_all_pin_types(&self, pool: &PgPool) -> Result<Vec<PinType>, Box<dyn Error>> {
        if let Some(cached_pin_types) = self.pin_types_cache.get(PIN_TYPE_CACHE_KEY).await {
            return Ok(cached_pin_types);
        }

        let pin_types: &Vec<PinType> =
            &(match sqlx::query_as!(PinType, r#"SELECT * FROM pin_types;"#)
                .fetch_all(pool)
                .await
            {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            });

        self.pin_types_cache
            .insert(String::from(PIN_TYPE_CACHE_KEY), pin_types.clone())
            .await;

        Ok(pin_types.clone())
    }

    pub async fn get_all_pin(&self, pool: &PgPool) -> Result<Vec<DBPin>, Box<dyn Error>> {
        if let Some(cached_pin_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_pin_list);
        }

        let pins: &Vec<DBPin> = &(match sqlx::query_as!(DBPin, r#"SELECT * FROM pins;"#)
            .fetch_all(pool)
            .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        Ok(pins.clone())
    }
}
