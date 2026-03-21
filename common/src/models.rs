use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericSensorReading {
    pub time: Option<DateTime<Utc>>,
    pub sensor_id: Uuid,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SenMLRecord {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "u", skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "v")]
    pub value: f64,
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl Display for SenMLRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dt = chrono::DateTime::from_timestamp(self.timestamp.unwrap_or(0), 0).unwrap();
        write!(
            f,
            "[{}] {} - {} {}",
            dt,
            self.name,
            self.value,
            self.unit.as_ref().unwrap_or(&"".to_string())
        )
    }
}
