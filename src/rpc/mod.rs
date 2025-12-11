use anyhow::{anyhow, Result};
use ethers::prelude::*;
use std::sync::Arc;
use tracing::error;

use crate::models::BlockData;



#[derive(Debug, Clone)]
pub struct BlockchainClient {
    http_client: Arc<Provider<Http>>,
}

impl BlockchainClient {
    pub fn new(http_url: &str, _ws_url: &str) -> Result<Self> {
        let http_provider = Provider::<Http>::try_from(http_url)?;
        
        Ok(Self {
            http_client: Arc::new(http_provider),
        })
    }

    pub async fn get_current_block_number(&self) -> Result<u64> {
        let block_number = self.http_client.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    pub async fn fetch_block_batch(&self, start_block: u64, count: usize) -> Result<Vec<BlockData>> {
        let mut blocks = Vec::with_capacity(count);
        let mut futures = Vec::new();

        for i in 0..count {
            let block_number = start_block + i as u64;
            let client = Arc::clone(&self.http_client);
            
            let future = tokio::spawn(async move {
                Self::fetch_single_block(client, block_number).await
            });
            
            futures.push(future);
        }

        for future in futures {
            match future.await? {
                Ok(block_data) => blocks.push(block_data),
                Err(e) => {
                    error!("Failed to fetch block: {}", e);
                    return Err(e);
                }
            }
        }

        blocks.sort_by_key(|b| b.number);
        Ok(blocks)
    }

    pub async fn fetch_single_block_data(&self, block_number: u64) -> Result<BlockData> {
        Self::fetch_single_block(Arc::clone(&self.http_client), block_number).await
    }

    async fn fetch_single_block(client: Arc<Provider<Http>>, block_number: u64) -> Result<BlockData> {
        let block = client
            .get_block_with_txs(BlockNumber::Number(block_number.into()))
            .await?
            .ok_or_else(|| anyhow!("Block {block_number} not found"))?;

        let transactions: Vec<String> = block
            .transactions
            .iter()
            .map(|tx| format!("{:?}", tx.hash))
            .collect();

        Ok(BlockData {
            number: block.number.unwrap().as_u64(),
            timestamp: block.timestamp.as_u64(),
            transactions,
        })
    }

}