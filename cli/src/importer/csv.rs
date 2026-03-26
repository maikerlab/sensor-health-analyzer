use chrono::{DateTime, Utc};
use sensorvault_core::models::{CreateSensor, CreateSensorData};
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct CsvRecord {
    timestamp: DateTime<Utc>,
    sensor_id: String,
    channel: String,
    unit: String,
    value: f64,
}

impl Into<CreateSensorData> for CsvRecord {
    fn into(self) -> CreateSensorData {
        CreateSensorData {
            time: self.timestamp,
            sensor_id: self.sensor_id,
            value: self.value,
        }
    }
}

impl Into<CreateSensor> for CsvRecord {
    fn into(self) -> CreateSensor {
        CreateSensor {
            id: self.sensor_id,
            channel: self.channel,
            unit: Some(self.unit),
            description: Some("imported by csv".to_string()),
        }
    }
}

pub fn parse_sensor_data_from_csv(path: &Path) -> anyhow::Result<Vec<CsvRecord>> {
    let file = File::open(path)?;
    let mut sensor_readings: Vec<CsvRecord> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: CsvRecord = result?;
        sensor_readings.push(record);
    }

    Ok(sensor_readings)
}
