use anyhow::Result;
use common::settings::Settings;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

pub mod data;
pub mod registry;

pub async fn connect_postgres() -> Result<PgPool> {
    let db_url = Settings::load()
        .get_database_url_or_default();
    info!("connecting to postgres: {}", db_url.as_str());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;

    Ok(pool)
}
