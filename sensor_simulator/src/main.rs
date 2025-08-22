use std::time::Duration;
use clap::{Parser, Subcommand};
use rumqttc::{Client, Connection, MqttOptions, QoS};
use tracing::{debug, info};
use common::settings::Settings;
use anyhow::Result;
use rand::Rng;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// does testing things
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
        sensor_type: String,
    }
}

fn handle_send_msg(mqtt_client: &Client, connection: &mut Connection, topic: &str, payload: &str) {
    println!("Sending message: {payload}");
    mqtt_client.publish(topic, QoS::AtLeastOnce, true, payload.as_bytes())
        .expect("error publishing messasge");
    connection.iter().take(3).for_each(drop);
}

fn handle_send_loop(mqtt_client: &Client, connection: &mut Connection, sensor_type: &str) {
    let mut rng = rand::rng();
    loop {
        let sensor_id: i32 = rng.random_range(1..10);
        let value: f64 = rng.random_range(10.0..35.0);
        let topic = format!("sensors/{}/{}", sensor_type, sensor_id);
        let payload = format!("{:.1}", value);
        info!("Publish to {} -> {}", topic, payload);
        mqtt_client.publish(topic, QoS::AtLeastOnce, true, payload.as_bytes())
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

    let settings = Settings::load();
    let host = settings.mqtt_host.unwrap_or("localhost".to_string());
    let port = settings.mqtt_port.unwrap_or(1883);
    info!("Connecting to MQTT broker: {}:{}", host, port);
    let mut mqttoptions = MqttOptions::new("sensor-simulator", host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut connection) = Client::new(mqttoptions, 10);

    match &cli.command {
        Some(Commands::Send { topic, msg, times }) => {
            for _ in 0..times.unwrap_or(1) {
                handle_send_msg(&client, &mut connection, topic.as_str(), msg.as_str());
            }
        }
        Some(Commands::Loop { sensor_type }) => {
            handle_send_loop(&client, &mut connection, sensor_type.as_str());
        }
        None => {},
    }

    Ok(())
}
