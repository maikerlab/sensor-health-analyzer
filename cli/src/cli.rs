use std::path::Path;
use clap::{Parser, Subcommand};
use sensorvault_core::models::{CreateSensor, CreateSensorData};
use sensorvault_infra::persistence::postgres::PostgresDatabase;
use sensorvault_infra::persistence::{SensorDataRepository, SensorRepository};
use crate::importer::csv::parse_sensor_data_from_csv;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// imports sensor data from file
    Import {
        /// path to the file to import
        file_path: String,
    },
    /// finds saved sensor data by sensor id
    Find { sensor_id: String },
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub async fn run(&self, db: PostgresDatabase) -> anyhow::Result<()> {
        match &self.command {
            None => {}
            Some(Commands::Import { file_path }) => {
                let csv_records = parse_sensor_data_from_csv(Path::new(file_path.as_str()))?;
                for record in &csv_records {
                    let sensor: CreateSensor = record.clone().into();
                    let sensor_data: CreateSensorData = record.clone().into();
                    db.save_sensor(&sensor).await.ok();
                    match db.save_sensor_reading(&sensor_data).await {
                        Ok(saved) => {
                            println!("Saved: {:?}", saved);
                        }
                        Err(e) => {
                            println!("Failed to save sensor data: {}", e);
                        }
                    }
                }
                println!("FINISHED -> Imported {} record(s)!", csv_records.len());
            }
            Some(Commands::Find { sensor_id }) => {
                let readings = db
                    .find_readings_by_sensor_id(sensor_id.as_str())
                    .await
                    .expect("Failed to find readings");
                for reading in readings {
                    println!("Found reading: {:?}", reading);
                }
            }
        }
        Ok(())
    }
}