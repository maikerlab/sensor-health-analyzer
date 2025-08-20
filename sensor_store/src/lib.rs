use crate::models::Sensor;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use common::SenMLRecord;

mod models;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let db_url = env::var("DATABASE_URL").expect("database url must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;

    Ok(pool)
}

pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng; // secure random number generator
    let private_key = SigningKey::generate(&mut csprng);
    let public_key = private_key.verifying_key();
    (private_key, public_key)
}

pub async fn register_sensor(
    pool: &PgPool,
    id: i32,
    sensor_type: &str,
    location: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO sensors (id, type, location)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO NOTHING
        "#,
        id,
        sensor_type,
        location,
    )
        .execute(pool)
        .await?;

    let row: (i64,) = sqlx::query_as("SELECT * FROM sensors WHERE id = $1")
        .bind(id)
        .bind(123)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn get_sensor_by_id(pool: &PgPool, id: i32) -> Result<Sensor, sqlx::Error> {
    let sensor = sqlx::query_as!(
        Sensor,
        r#"SELECT * FROM sensors WHERE id = $1"#,
        id,
    )
        .fetch_one(pool)
        .await?;
    Ok(sensor)
}

pub async fn save_sensor_data(pool: &PgPool, record: SenMLRecord, sensor_id: i32) -> Result<(), sqlx::Error> {
    let ts = chrono::DateTime::from_timestamp(record.timestamp.unwrap_or(chrono::Utc::now().timestamp()), 0);
    sqlx::query!(
        r#"
        INSERT INTO sensor_data (time, sensor_id, value, unit)
        VALUES ($1, $2, $3, $4)
        "#,
        ts,
        sensor_id,
        record.value,
        record.unit
    )
        .execute(pool)
        .await?;

   Ok(())
}