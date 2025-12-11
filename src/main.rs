mod config;
mod db;
mod ingestion;
mod models;
mod rpc;

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use config::Config;
use db::Database;
use ingestion::IngestionService;
use rpc::BlockchainClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string())
                .as_str(),
        )
        .init();

    info!("Starting Intuition Fast Ingestion Service");

    let config = Config::from_env()?;
    info!("Configuration loaded");

    let database = Arc::new(
        Database::new(&config.database_url, config.db_max_connections).await?,
    );
    info!("Database connection established and migrations applied");

    let blockchain_client = Arc::new(
        BlockchainClient::new(&config.rpc_http_url, &config.rpc_ws_url)?,
    );
    info!("Blockchain client initialized");

    let ingestion_service = IngestionService::new(
        Arc::clone(&database),
        Arc::clone(&blockchain_client),
        config.batch_size,
    );

    info!("Starting ingestion service...");
    
    if let Err(e) = ingestion_service.start().await {
        error!("Ingestion service failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}