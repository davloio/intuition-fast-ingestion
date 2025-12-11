use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub rpc_http_url: String,
    pub rpc_ws_url: String,
    pub batch_size: usize,
    pub db_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/intuition_indexer".to_string());

        let rpc_http_url = env::var("RPC_HTTP_URL")
            .unwrap_or_else(|_| "https://rpc.intuition.systems".to_string());

        let rpc_ws_url = env::var("RPC_WS_URL")
            .unwrap_or_else(|_| "wss://rpc.intuition.systems".to_string());

        let batch_size = env::var("BATCH_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .unwrap_or(1000);

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .unwrap_or(20);

        Self {
            database_url,
            rpc_http_url,
            rpc_ws_url,
            batch_size,
            db_max_connections,
        }
    }
}