use sqlx::Pool;
use tracing::info;

use crate::error::DbError;

pub async fn run(pool: &Pool<sqlx::Sqlite>) -> Result<(), DbError> {
    info!("Running database migrations");

    // Create containers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS containers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'stopped',
            template TEXT NOT NULL,
            node_id TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            config TEXT NOT NULL,
            CONSTRAINT valid_status CHECK (status IN ('stopped', 'running', 'starting', 'stopping', 'frozen', 'error'))
        )
        "#
    )
        .execute(pool)
        .await?;

    // Create index on name for faster lookups
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_containers_name ON containers(name)")
        .execute(pool)
        .await?;

    // Create index on status for filtering
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_containers_status ON containers(status)")
        .execute(pool)
        .await?;

    // Create index on created_at for sorting
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_containers_created_at ON containers(created_at DESC)")
        .execute(pool)
        .await?;

    info!("Database migrations completed successfully");
    Ok(())
}
