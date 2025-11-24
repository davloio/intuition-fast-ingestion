use anyhow::{anyhow, Result};
use ethers::prelude::*;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::models::BlockData;

pub mod client;
pub mod websocket;


#[derive(Debug, Clone)]
pub struct BlockchainClient {
    http_client: Arc<Provider<Http>>,
    ws_url: String,
}

impl BlockchainClient {
    pub async fn new(http_url: &str, ws_url: &str) -> Result<Self> {
        let http_provider = Provider::<Http>::try_from(http_url)?;
        
        Ok(Self {
            http_client: Arc::new(http_provider),
            ws_url: ws_url.to_string(),
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
            .ok_or_else(|| anyhow!("Block {} not found", block_number))?;

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

    pub async fn start_live_subscription(&self) -> Result<mpsc::Receiver<BlockData>> {
        let ws_url = self.ws_url.clone();
        let http_client = Arc::clone(&self.http_client);
        
        let (tx, rx) = mpsc::channel::<BlockData>(100);

        tokio::spawn(async move {
            info!("Starting live block subscription");
            
            let ws_provider = match Provider::<Ws>::connect(&ws_url).await {
                Ok(provider) => provider,
                Err(e) => {
                    error!("Failed to connect WebSocket: {}", e);
                    return;
                }
            };
            
            let mut stream = match ws_provider.subscribe_blocks().await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Failed to subscribe to blocks: {}", e);
                    return;
                }
            };
            
            info!("Started live block subscription");
            
            while let Some(block) = stream.next().await {
                let block_number = block.number.unwrap().as_u64();
                
                debug!("Received new block: {}", block_number);
                
                match Self::fetch_single_block(
                    Arc::clone(&http_client),
                    block_number
                ).await {
                    Ok(block_data) => {
                        if let Err(e) = tx.send(block_data).await {
                            error!("Failed to send block data: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch block {}: {}", block_number, e);
                    }
                }
            }
            
            warn!("Live block subscription ended");
        });

        Ok(rx)
    }
}