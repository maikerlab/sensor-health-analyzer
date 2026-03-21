use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::types::Uuid;
use std::fmt::Display;

#[derive(FromRow)]
pub struct Device {
    pub id: Uuid,
    pub material_no: Option<String>,
    pub serial_no: Option<String>,
    pub custom_id: Option<String>,
    pub name: String,
    pub device_type: Option<String>,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct Sensor {
    pub id: Uuid,
    pub custom_id: Option<String>,
    pub device_id: Option<Uuid>,
    pub channel: String,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SensorData {
    pub time: DateTime<Utc>,
    pub sensor_id: Uuid,
    pub value: f64,
}

impl Display for SensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{}] id={} value={}",
            self.time.format("%d.%m.%Y %H:%M:%S"), self.sensor_id, self.value
        ))
    }
}
