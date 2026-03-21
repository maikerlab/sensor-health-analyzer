pub mod models;

use crate::db::models::{Sensor, SensorData};
use chrono::Utc;
use common::models::GenericSensorReading;
use common::settings::DatabaseSettings;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;
use sqlx::PgPool;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: DatabaseSettings) -> anyhow::Result<Self> {
        let url = config.url.as_str();
        let pool = PgPoolOptions::new().max_connections(3).connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn get_sensor_by_custom_id(
        &self,
        custom_id: &str,
    ) -> anyhow::Result<Sensor> {
        let sensor = sqlx::query_as!(
            Sensor,
            r#"
                SELECT id, custom_id, device_id, channel, unit, description, created_at
                FROM sensors
                WHERE custom_id = $1
            "#,
            custom_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(sensor)
    }

    pub async fn save_sensor(&self, dto: Sensor) -> anyhow::Result<Sensor> {
        let sensor = sqlx::query_as!(
            Sensor,
            r#"
                INSERT INTO sensors (custom_id, device_id, channel, unit, description)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            "#,
            dto.custom_id,
            dto.device_id,
            dto.channel,
            dto.unit,
            dto.description
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(sensor)
    }

    pub async fn save_sensor_reading(
        &self,
        reading: &GenericSensorReading,
        sensor_id: &Uuid,
    ) -> anyhow::Result<SensorData> {
        let sensor_data = sqlx::query_as!(
            SensorData,
            r#"
                INSERT INTO sensor_data (time, sensor_id, value)
                VALUES ($1, $2, $3)
                RETURNING *
            "#,
            reading.time.unwrap_or(Utc::now()),
            sensor_id,
            reading.value
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(sensor_data)
    }

    pub async fn get_sensor_readings_by_sensor_id(
        &self,
        sensor_id: Uuid,
    ) -> anyhow::Result<Vec<SensorData>> {
        let sensor_data = sqlx::query_as!(
            SensorData,
            r#"
                SELECT time, sensor_id, value
                FROM sensor_data
                WHERE sensor_id = $1
                ORDER BY time DESC
            "#,
            sensor_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(sensor_data)
    }
}
