use anyhow::Result;
use async_trait::async_trait;
use crate::models::{Block, Transaction, IngestionState, IngestionMode};

#[async_trait]
pub trait BlockRepository {
    async fn get_latest_block(&self) -> Result<Option<i64>>;
    async fn batch_insert_blocks(&self, blocks: &[Block]) -> Result<()>;
}

#[async_trait]
pub trait TransactionRepository {
    async fn batch_insert_transactions(&self, transactions: &[Transaction]) -> Result<()>;
}

#[async_trait]
pub trait StateRepository {
    async fn get_state(&self) -> Result<IngestionState>;
    async fn update_state(&self, last_block: i64, mode: IngestionMode) -> Result<()>;
}