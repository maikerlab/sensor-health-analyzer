use common::SenMLRecord;
use futures::StreamExt;
use db::data::save_record;
use tracing::{error, info};
use anyhow::Result;

const STREAM_NAME_TEMPERATURE: &str = "TEMPERATURE_SENSOR_EVENTS";

#[tokio::main]
async fn main() -> Result<()> {
    // init logging
    tracing_subscriber::fmt::init();

    let db = db::connect_postgres().await?;
    let nats = messaging::connect_nats().await?;

    let stream_name = String::from(STREAM_NAME_TEMPERATURE);
    let subjects = vec!["temp.>".into()];
    let consumer = messaging::subscribe(&nats, stream_name, subjects).await?;
    // Attach to the messages iterator for the Consumer.
    let mut messages = consumer.messages().await?;

    // Iterate over messages.
    info!("waiting for messages from workers...");
    while let Some(message) = messages.next().await {
        let message = message?;

        // parse sensor_id as last part of subject
        let sensor_id_str = message.subject.split('.').last().expect("cannot parse sensor_id");
        let sensor_id = sensor_id_str.parse::<i32>().expect("cannot parse sensor_id");

        // Deserialize from CBOR - also support JSON as a fallback
        let record: SenMLRecord = serde_cbor::from_slice(&message.payload)
            .or_else(|_| serde_json::from_slice(&message.payload))
            .expect("error at CBOR or JSON deserialization");

        match save_record(&db, record.clone(), sensor_id).await {
            Ok(_) => info!("Saved sensor_data: {}", record),
            Err(e) => error!("Error saving sensor_data: {}", e),
        }

        // acknowledge the message
        message.ack().await.map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(())
}
