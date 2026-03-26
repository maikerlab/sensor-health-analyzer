use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct SensorData {
    pub time: DateTime<Utc>,
    pub sensor_id: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct CreateSensorData {
    pub time: DateTime<Utc>,
    pub sensor_id: String,
    pub value: f64,
}

#[derive(Debug)]
pub struct Sensor {
    pub id: String,
    pub device_id: Option<Uuid>,
    pub channel: String,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateSensor {
    pub id: String,
    pub channel: String,
    pub unit: Option<String>,
    pub description: Option<String>,
}
