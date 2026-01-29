use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Pool;
use std::path::Path;
use tracing::info;

use crate::error::DbError;
use crate::migrations;

pub type DbPool = Pool<sqlx::Sqlite>;

pub struct PoolConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:///tmp/arm-hypervisor.db".to_string(),
            max_connections: 10,
        }
    }
}

pub async fn create_pool(config: PoolConfig) -> Result<DbPool, DbError> {
    info!("Creating database pool at: {}", config.database_url);

    // Ensure the database file directory exists
    if let Some(parent) = Path::new(config.database_url.strip_prefix("sqlite://").unwrap_or("")).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    // Create the pool
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    migrations::run(&pool).await?;

    Ok(pool)
}
