use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::models::BlockData;

#[async_trait]
pub trait WebSocketClient {
    async fn subscribe_new_blocks(&self) -> Result<mpsc::Receiver<BlockData>>;
}