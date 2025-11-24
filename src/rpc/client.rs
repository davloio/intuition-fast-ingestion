use anyhow::Result;
use async_trait::async_trait;
use crate::models::BlockData;

#[async_trait]
pub trait RpcClient {
    async fn get_latest_block_number(&self) -> Result<u64>;
    async fn fetch_blocks(&self, start: u64, count: usize) -> Result<Vec<BlockData>>;
    async fn fetch_block(&self, number: u64) -> Result<BlockData>;
}