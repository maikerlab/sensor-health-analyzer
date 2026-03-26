pub mod decoder;
mod input;

use crate::ingestion::decoder::{DecodedSensorReading, DecoderRegistry};
use crate::ingestion::input::RawInput;
use chrono::Utc;
use rumqttc::Packet::Publish;
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use sensorvault_core::models::{CreateSensor, CreateSensorData, Sensor};
use sensorvault_infra::persistence::{SensorDataRepository, SensorRepository};
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct IngestionService<R>
where
    R: SensorRepository + SensorDataRepository,
{
    db: R,
    decoder_registry: DecoderRegistry,
}

impl<R> IngestionService<R>
where
    R: SensorRepository + SensorDataRepository,
{
    pub fn new(db: R, decoder_registry: DecoderRegistry) -> Self {
        Self {
            db,
            decoder_registry,
        }
    }

    pub async fn run(&self, mqtt_host: String, mqtt_port: u16) -> anyhow::Result<()> {
        let mut mqttoptions = MqttOptions::new("sha-collector", mqtt_host.as_str(), mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        client.subscribe("sensors/+/+", QoS::AtLeastOnce).await?;

        info!("Running Ingestion service");
        loop {
            let event = eventloop.poll().await?;
            match event {
                Event::Incoming(Publish(packet)) => {
                    let input = RawInput::Mqtt {
                        topic: packet.topic.to_string(),
                        payload: packet.payload.to_vec(),
                    };
                    self.process(input).await;
                }
                _ => {}
            }
        }
    }

    pub async fn process(&self, input: RawInput) {
        if let Err(e) = self.try_process(&input).await {
            warn!(
                error = %e,
                source = %input_label(&input),
                "Failed to process input – skipping"
            );
        }
    }

    async fn try_process(&self, input: &RawInput) -> anyhow::Result<()> {
        let readings = self.decoder_registry.decode(input)?;

        if readings.is_empty() {
            debug!(
                source = %input_label(input),
                "Decoder returned no readings – skipping"
            );
            return Ok(());
        }

        for reading in readings {
            self.persist(reading).await?;
        }

        Ok(())
    }

    async fn persist(&self, reading: DecodedSensorReading) -> anyhow::Result<()> {
        debug!(
            reading = %reading,
            "Persisting reading"
        );
        let sensor = self.resolve_sensor(&reading).await?;

        let row = CreateSensorData {
            time: Utc::now(),
            sensor_id: sensor.id.clone(),
            value: reading.value,
        };

        self.db.save_sensor_reading(&row).await?;

        info!(
            sensor_id = %sensor.id,
            topic = reading.channel,
            value     = reading.value,
            "Reading persisted"
        );

        Ok(())
    }

    async fn resolve_sensor(&self, reading: &DecodedSensorReading) -> anyhow::Result<Sensor> {
        if let Some(sensor) = self.db.find_sensor_by_id(reading.id.as_str()).await? {
            return Ok(sensor);
        }
        let create_sensor = CreateSensor {
            id: reading.id.clone(),
            channel: reading.channel.clone(),
            unit: reading.unit.clone(),
            description: None,
        };
        self.db.save_sensor(&create_sensor).await
    }
}

/// Helper for readable log messages – no logic here
pub fn input_label(input: &RawInput) -> String {
    match input {
        RawInput::Mqtt { topic, .. } => format!("mqtt:{topic}"),
        RawInput::Manual {
            material_no,
            serial_no,
            channel,
            ..
        } => format!("manual:{material_no}/{serial_no}/{channel}"),
    }
}
