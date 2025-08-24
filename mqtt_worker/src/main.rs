use async_nats::jetstream::Context;
use chrono::Utc;
use common::SenMLRecord;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::time::Duration;
use tracing::{debug, error, info};
use common::settings::Settings;
use anyhow::{anyhow, Result};

async fn publish_measurement(context: &Context, sensor_id: i32, record: SenMLRecord) -> Result<()> {
    let payload = serde_cbor::to_vec(&record)?;
    context.publish(format!("temp.{}", sensor_id), payload.into())
        .await? // publish
        .await?; // await acknowledgement
    info!("SenMLRecord successfully sent to NATS: {}", record);
    Ok(())
}

async fn handle_message(ctx: &Context, topic: &str, payload: &str) -> Result<()> {
    debug!("{} - {}", topic, payload);
    let sensor_type = topic.split('/').nth(1).unwrap();
    let sensor_id = topic.split('/').nth(2).unwrap();
    let sensor_id = sensor_id.parse::<i32>().unwrap();
    let value = payload.parse::<f64>().unwrap();
    let unit = match sensor_type {
        "temp" => "C",
        "humidity" => "%RF",
        _ => "unknown"
    };
    info!("Got payload {} {} from sensor {}-{}", payload, unit, sensor_type, sensor_id);
    let msg = SenMLRecord {
        name: format!("{}-{}", sensor_type, sensor_id),
        value,
        unit: Some(unit.to_string()),
        timestamp: Some(Utc::now().timestamp())
    };
    publish_measurement(&ctx, sensor_id, msg).await?;
    Ok(())
}

async fn handle_event(nats: &Context, event: Event) -> Result<()> {
    match event {
        Event::Incoming(Incoming::Publish(p)) => {
            debug!("Received at {} = {:?}", p.topic.clone(), p.payload.clone());
            if let Ok(payload_str) = str::from_utf8(&p.payload) {
                handle_message(&nats, p.topic.as_str(), payload_str).await?;
            }
        },
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // init logging
    tracing_subscriber::fmt::init();

    // Connect to the NATS server
    let settings = Settings::load();
    let nats = messaging::connect_nats().await.
        expect("Failed to connect to NATS - is the server running and the URL set correctly?");

    let (mqtt_host, mqtt_port) = settings.get_mqtt_host_and_port_or_default();
    let mut mqttoptions = MqttOptions::new("rumqtt-async", mqtt_host, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("sensors/+/+", QoS::AtLeastOnce).await?;

    info!("Waiting for MQTT messages...");
    loop {
        let event = eventloop.poll().await.map_err(|e| {
            anyhow!("Error polling event: {}", e)
        })?;
        let _ = handle_event(&nats, event).await.map_err(|e|
            error!("Error handling event: {}", e)
        );
    }
}
