use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Sensor {
    pub id: i32,
    pub r#type:  Option<String>,
    pub location: Option<String>,
    // pub name: String,
    // pub type_id: Uuid,
    // pub public_key: Vec<u8>,
    // pub status: String,
    // pub registered_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SensorData {
    pub timestamp: DateTime<Utc>,
    pub sensor_id: Option<i32>,
    pub value: Option<f64>,
    pub unit: Option<f64>,
}
