use std::{error::Error, time::Duration};

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, Transaction};

const CACHE_KEY: &str = "PIN_CACHE";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBPin {
    pub id: BigDecimal,
    pub name: String,
    pub type_id: BigDecimal,
    pub lat: f32,
    pub long: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub address: String,
    pub is_sponsored: bool,
    pub terms: String,
    pub instruction: String,
    pub opening: Vec<i32>,
    pub closing: Vec<i32>,
    pub image_id: BigDecimal,
    pub accepts: String,
    pub opening_days: i16,
    pub note: String,
}

pub struct PinRepo {
    cache: Cache<String, Vec<DBPin>>,
}

impl PinRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
        }
    }

    pub async fn get_all_pin(
        &self,
        pool: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<DBPin>, Box<dyn Error>> {
        if let Some(cached_pin_list) = self.cache.get(CACHE_KEY).await {
            return Ok(cached_pin_list);
        }

        let pins: &Vec<DBPin> = &(match sqlx::query_as!(DBPin, r#"SELECT * FROM pins;"#)
            .fetch_all(&mut **pool)
            .await
        {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });

        self.cache
            .insert(String::from(CACHE_KEY), pins.clone())
            .await;

        Ok(pins.clone())
    }

    pub async fn create_pin(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        name: &str,
        type_id: u128,
        lat: f32,
        long: f32,
        address: &str,
        is_sponsored: bool,
        terms: &str,
        opening: Vec<i32>,
        closing: Vec<i32>,
        instruction: &str,
        image_id: u128,
        accepts: &str,
        opening_days: i16,
        note: &str,
    ) -> Result<DBPin, Box<dyn Error>> {
        let pin: &DBPin = &(match sqlx::query_as!(
                    DBPin,
                    r#"INSERT INTO pins (id, name, type_id, lat, long, address, is_sponsored, terms, opening, closing, instruction, image_id, accepts, opening_days, note) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING *;"#,
                    BigDecimal::from_u128(id).unwrap_or_default(),
                    name,
                    BigDecimal::from_u128(type_id).unwrap_or_default(),
                    lat,
                    long,
                    address,
                    is_sponsored,
                    terms,
                    &opening,
                    &closing,
                    instruction,
                    BigDecimal::from_u128(image_id).unwrap_or_default(),
                    &accepts,
                    opening_days,
                    &note
                ).fetch_one(&mut **pool).await {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        });
        if let Some(mut cached_pin_list) = self.cache.get(CACHE_KEY).await {
            cached_pin_list.push(pin.clone());
            self.cache
                .insert(String::from(CACHE_KEY), cached_pin_list)
                .await;
        }
        Ok(pin.clone())
    }
}
