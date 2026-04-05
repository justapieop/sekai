use std::time::Duration;

use chrono::Utc;
use tokio::time::sleep;

pub struct SnowflakeGenerator {
    last_timestamp: u64,
    machine_id: u32,
    sequence: u32,
}

#[derive(Debug, Clone)]
pub struct Snowflake {
    pub timestamp: u64,
    pub machine_id: u32,
    pub sequence: u32,
    pub id: u128,
}

impl SnowflakeGenerator {
    pub fn new(machine_id: u32) -> Self {
        Self {
            last_timestamp: Utc::now().timestamp_millis() as u64,
            machine_id,
            sequence: 0,
        }
    }

    pub async fn next_id(&mut self) -> Snowflake {
        let mut current_timestamp: u64 = Utc::now().timestamp_millis() as u64;

        if current_timestamp.eq(&self.last_timestamp) {
            let _ = self.sequence.wrapping_add(1);
            let seq_mask = (1_u128 << 32) - 1;
            if ((self.sequence as u128) & seq_mask) == 0 {
                sleep(Duration::from_millis(1)).await;
                current_timestamp = Utc::now().timestamp_millis() as u64;
                self.sequence = 0;
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = current_timestamp;

        let timestamp_128 = u128::from(current_timestamp) & ((1_u128 << 64) - 1);
        let machine_id_128 = u128::from(self.machine_id) & ((1_u128 << 32) - 1);
        let sequence_128 = u128::from(self.sequence) & ((1_u128 << 32) - 1);

        Snowflake {
            timestamp: current_timestamp,
            machine_id: self.machine_id,
            sequence: self.sequence,
            id: (timestamp_128 << 64) | (machine_id_128 << 32) | sequence_128,
        }
    }

    pub fn from(snowflake_id: u128) -> Snowflake {
        let timestamp: u64 = (snowflake_id >> 64).try_into().unwrap_or_default();
        let machine_id: u32 = ((snowflake_id >> 32) & 0xFFFFFFFF)
            .try_into()
            .unwrap_or_default();

        let sequence: u32 = (snowflake_id & 0xFFFFFFFF).try_into().unwrap_or_default();

        Snowflake {
            timestamp,
            machine_id,
            sequence,
            id: snowflake_id,
        }
    }
}
