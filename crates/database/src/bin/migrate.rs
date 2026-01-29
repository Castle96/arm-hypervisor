use database::pool::{create_pool, PoolConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = PoolConfig {
        database_url: "sqlite:///tmp/arm-hypervisor.db".to_string(),
        max_connections: 10,
    };

    let _pool = create_pool(config).await?;
    info!("Database migrations completed!");

    Ok(())
}
