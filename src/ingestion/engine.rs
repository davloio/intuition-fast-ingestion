use anyhow::Result;
use async_trait::async_trait;
use crate::models::{BlockData, IngestionMode};

#[async_trait]
pub trait IngestionEngine {
    async fn detect_mode(&self) -> Result<IngestionMode>;
    async fn process_reindex_batch(&self, start_block: u64, count: usize) -> Result<Vec<BlockData>>;
    async fn process_live_block(&self, block_number: u64) -> Result<BlockData>;
    async fn start_live_subscription(&self) -> Result<()>;
}