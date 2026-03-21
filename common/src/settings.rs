use config::FileFormat::Toml;
use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub database: DatabaseSettings,
    pub mqtt: MqttSettings,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MqttSettings {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn load() -> Self {
        let settings_file = File::new("settings.toml", Toml);
        let settings = Config::builder()
            .add_source(settings_file.required(false))
            .add_source(config::Environment::with_prefix("SHA")
                .try_parsing(true)
                .separator("_"))
            .set_default("database.url", "postgres://iot:sensor@localhost/sensor_db").unwrap()
            .set_default("mqtt.host", "localhost").unwrap()
            .set_default("mqtt.port", 1883).unwrap()
            .build()
            .expect("Failed to load settings");
        settings
            .try_deserialize()
            .expect("Failed to parse settings")
    }

}
