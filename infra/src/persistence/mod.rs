use sensorvault_core::models::CreateSensor;
use sensorvault_core::models::CreateSensorData;
use sensorvault_core::models::Sensor;
use sensorvault_core::models::SensorData;

pub mod postgres;
mod models;

pub trait SensorRepository {
    fn find_sensor_by_id(&self, topic: &str) -> impl Future<Output = anyhow::Result<Option<Sensor>>> + Send;
    fn save_sensor(&self, sensor: CreateSensor) -> impl Future<Output = anyhow::Result<Sensor>> + Send;
}

pub trait SensorDataRepository {
    fn find_readings_by_sensor_id(&self, sensor_id: &str) -> impl Future<Output = anyhow::Result<Vec<SensorData>>> + Send;
    fn find_readings_by_sensor_id_since_minutes(&self, sensor_id: &str, since_minutes: i32) -> impl Future<Output = anyhow::Result<Vec<SensorData>>> + Send;
    fn save_sensor_reading(&self, reading: &CreateSensorData) -> impl Future<Output = anyhow::Result<SensorData>> + Send;
}

