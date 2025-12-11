use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Block {
    pub number: i64,
    pub timestamp: i64,
    pub transaction_count: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Transaction {
    pub hash: String,
    pub block_number: i64,
    pub position: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IngestionState {
    #[allow(dead_code)]
    pub id: i32,
    pub last_processed_block: i64,
    pub mode: String,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BlockData {
    pub number: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IngestionMode {
    Reindex,
    Live,
}

impl From<String> for IngestionMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "live" => IngestionMode::Live,
            _ => IngestionMode::Reindex,
        }
    }
}

impl From<IngestionMode> for String {
    fn from(mode: IngestionMode) -> Self {
        match mode {
            IngestionMode::Reindex => "reindex".to_string(),
            IngestionMode::Live => "live".to_string(),
        }
    }
}