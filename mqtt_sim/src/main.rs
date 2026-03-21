use anyhow::Result;
use clap::{Parser, Subcommand};
use common::settings::AppConfig;
use rand::prelude::IndexedRandom;
use rand::{RngExt};
use rumqttc::{Client, Connection, MqttOptions, QoS};
use std::time::Duration;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// send a message to the worker(s)
    Send {
        /// topic to send to
        topic: String,
        /// message to send
        msg: String,
        /// number of times to send
        times: Option<usize>,
    },
    /// sends measurements in an endless loop
    Loop {
        /// type of the sensor (temp or humidity)
        sensor_type: Option<String>,
    },
}

fn handle_send_msg(mqtt_client: &Client, connection: &mut Connection, topic: &str, payload: &str) {
    println!("Sending message: {payload}");
    mqtt_client
        .publish(topic, QoS::AtLeastOnce, true, payload.as_bytes())
        .expect("error publishing messasge");
    connection.iter().take(3).for_each(drop);
}

fn handle_send_loop(
    mqtt_client: &Client,
    connection: &mut Connection,
    sensor_type: Option<String>,
) {
    let available_types = vec!["temp", "humidity"];
    loop {
        let mut rng = rand::rng();
        let sensor_type = match &sensor_type {
            None => available_types.choose(&mut rand::rng()).unwrap(),
            Some(st) => st.as_str(),
        };
        let (sensor_id, value) = match sensor_type {
            "temp" => (rng.random_range(1..5), rng.random_range(-20.0..50.0)),
            "humidity" => (rng.random_range(6..10), rng.random_range(0.0..100.0)),
            _ => (1, 0.0),
        };
        let topic = format!("sensors/{}/{}", sensor_type, sensor_id);
        let payload = format!("{:.1}", value);
        info!("Publish to {} -> {}", topic, payload);
        mqtt_client
            .publish(topic, QoS::AtLeastOnce, true, payload.as_bytes())
            .expect("error publishing message");
        connection.iter().take(3).for_each(drop);
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn main() -> Result<()> {
    // init logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    debug!("{:?}", cli);

    let config = AppConfig::load();
    info!(
        "Connecting to MQTT broker: {}:{}",
        config.mqtt.host, config.mqtt.port
    );
    let mut mqttoptions = MqttOptions::new("mqtt-simulator", config.mqtt.host, config.mqtt.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut connection) = Client::new(mqttoptions, 10);

    match &cli.command {
        Commands::Send { topic, msg, times } => {
            for _ in 0..times.unwrap_or(1) {
                handle_send_msg(&client, &mut connection, topic.as_str(), msg.as_str());
            }
        }
        Commands::Loop { sensor_type } => {
            handle_send_loop(&client, &mut connection, sensor_type.clone());
        }
    }

    Ok(())
}
