use config::FileFormat::Toml;
use config::{Config, File};
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    pub database_url: Option<String>,
    pub nats_url: Option<String>,
    pub mqtt_host: Option<String>,
    pub mqtt_port: Option<u16>,
}

impl Settings {
    pub fn load() -> Self {
        let settings_file = File::new("settings.toml", Toml);
        let settings = Config::builder()
            .add_source(settings_file.required(false))
            .add_source(config::Environment::with_prefix("IOT"))
            .build()
            .expect("Failed to load settings");
        settings
            .try_deserialize()
            .expect("Failed to parse settings")
    }

    pub fn get_database_url_or_default(&self) -> String {
        self.database_url.clone().unwrap_or_else(|| {
            warn!("DATABASE_URL not set, using default");
            "postgres://iot:sensor@localhost/sensor_db".to_string()
        })
    }

    pub fn get_nats_url_or_default(&self) -> String {
        self.nats_url.clone().unwrap_or_else(|| {
            warn!("NATS_URL not set, using default");
            "nats://localhost:4222".to_string()
        })
    }

    pub fn get_mqtt_host_and_port_or_default(&self) -> (String, u16) {
        let mqtt_host = self.mqtt_host.clone().unwrap_or_else(|| {
            warn!("MQTT_HOST host not set, using default");
            "localhost".to_string()
        });
        let mqtt_port = self.mqtt_port.unwrap_or_else(|| {
            warn!("MQTT_PORT host not set, using default");
            1883
        });
        (mqtt_host, mqtt_port)
    }
}
