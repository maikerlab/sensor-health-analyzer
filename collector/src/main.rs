mod db;

use crate::db::models::SensorData;
use crate::db::{Database, models::Sensor};
use anyhow::{Result, anyhow};
use chrono::Utc;
use common::models::GenericSensorReading;
use common::settings::AppConfig;
use rumqttc::Packet::Publish;
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use sqlx::types::Uuid;
use std::str::from_utf8;
use std::time::Duration;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    // init logging
    tracing_subscriber::fmt::init();

    // Load config
    let settings = AppConfig::load();

    // Connect db
    let db = Database::new(settings.database).await?;

    let mut mqttoptions = MqttOptions::new("collector", settings.mqtt.host, settings.mqtt.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("sensors/+/+", QoS::AtLeastOnce).await?;

    info!("Waiting for MQTT messages...");
    loop {
        let event = eventloop
            .poll()
            .await
            .map_err(|e| anyhow!("Error polling event: {}", e))?;
        match event {
            Event::Incoming(Publish(packet)) => {
                let reading = save_mqtt_data(&db, packet.topic.as_str(), &packet.payload.to_vec())
                    .await?;
                info!("Saved new reading: {}", reading);
            }
            _ => {}
        }
    }
}

async fn save_mqtt_data(db: &Database, topic: &str, sensor_data: &Vec<u8>) -> Result<SensorData> {
    let v: Vec<&str> = topic.split("/").collect::<Vec<&str>>();
    let channel = if v.len() > 0 { v[1] } else { "temperature" };
    let mut sensor = db.get_sensor_by_custom_id(topic).await;
    if sensor.is_err() {
        sensor = db
            .save_sensor(Sensor {
                id: Uuid::new_v4(),
                custom_id: Some(topic.to_string()),
                device_id: None,
                channel: channel.to_string(),
                unit: Some("°C".to_string()),
                description: Some("MQTT reading".to_string()),
                created_at: Utc::now(),
            })
            .await;
    }
    let sensor = sensor?;
    let sensor_value = from_utf8(sensor_data)?;
    let value_float = sensor_value.parse::<f64>()?;
    debug!(
        "Received sensor data via MQTT - Topic: '{}', Value: {}",
        topic, sensor_value
    );
    let reading = GenericSensorReading {
        time: Some(Utc::now()),
        sensor_id: sensor.id,
        value: value_float,
    };
    db.save_sensor_reading(&reading, &sensor.id).await
}
