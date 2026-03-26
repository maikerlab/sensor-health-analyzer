mod cli;
mod importer;

use crate::cli::Cli;
use sensorvault_infra::persistence::postgres::PostgresDatabase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("Environment variable 'DATABASE_URL' must be set!"))?;
    let db = PostgresDatabase::connect(db_url, 1)
        .await
        .expect("Failed to connect to database");

    Cli::new().run(db).await?;

    Ok(())
}
