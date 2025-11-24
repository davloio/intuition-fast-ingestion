use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::db::Database;
use crate::models::{Block, BlockData, Transaction};
use crate::rpc::BlockchainClient;

pub mod engine;
pub mod buffer;


pub struct IngestionService {
    db: Arc<Database>,
    blockchain_client: Arc<BlockchainClient>,
    batch_size: usize,
}

impl IngestionService {
    pub fn new(
        db: Arc<Database>,
        blockchain_client: Arc<BlockchainClient>,
        batch_size: usize,
    ) -> Self {
        Self {
            db,
            blockchain_client,
            batch_size,
        }
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            match self.run_ingestion_cycle().await {
                Ok(_) => {
                    info!("Ingestion cycle completed successfully");
                }
                Err(e) => {
                    error!("Ingestion cycle failed: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn run_ingestion_cycle(&self) -> Result<()> {
        let state = self.db.get_ingestion_state().await?;
        let current_chain_height = self.blockchain_client.get_current_block_number().await?;
        let last_processed = state.last_processed_block as u64;

        info!(
            "Current state: last_processed={}, chain_height={}, mode={}",
            last_processed, current_chain_height, state.mode
        );

        let gap = current_chain_height.saturating_sub(last_processed);

        if gap == 0 {
            info!("Already at chain head, switching to live mode");
            return self.run_live_mode(last_processed).await;
        }

        if gap >= self.batch_size as u64 {
            info!("Gap is {}, running reindex mode", gap);
            return self.run_reindex_mode(last_processed, current_chain_height).await;
        }

        info!("Gap is {}, catching up remaining blocks", gap);
        self.catch_up_remaining_blocks(last_processed, current_chain_height).await?;
        self.run_live_mode(current_chain_height).await
    }

    async fn run_reindex_mode(&self, start_block: u64, target_block: u64) -> Result<()> {
        self.db
            .update_ingestion_state(start_block as i64, "reindex")
            .await?;

        let mut current_block = start_block + 1;

        while current_block + self.batch_size as u64 <= target_block {
            info!(
                "Fetching batch: blocks {} to {}",
                current_block,
                current_block + self.batch_size as u64 - 1
            );

            let block_data_batch = self
                .blockchain_client
                .fetch_block_batch(current_block, self.batch_size)
                .await?;

            self.process_and_store_blocks(block_data_batch).await?;

            current_block += self.batch_size as u64;

            self.db
                .update_ingestion_state(current_block as i64 - 1, "reindex")
                .await?;

            debug!("Updated state to block {}", current_block - 1);
        }

        let remaining = target_block - current_block + 1;
        if remaining > 0 {
            info!("Processing final {} blocks", remaining);
            return self.catch_up_remaining_blocks(current_block - 1, target_block).await;
        }

        Ok(())
    }

    async fn catch_up_remaining_blocks(&self, start_block: u64, target_block: u64) -> Result<()> {
        let mut current_block = start_block + 1;

        while current_block <= target_block {
            debug!("Fetching block {}", current_block);

            let block_data = self
                .blockchain_client
                .fetch_single_block_data(current_block)
                .await?;

            self.process_and_store_blocks(vec![block_data]).await?;

            self.db
                .update_ingestion_state(current_block as i64, "live")
                .await?;

            current_block += 1;
        }

        Ok(())
    }

    async fn run_live_mode(&self, start_block: u64) -> Result<()> {
        info!("Starting live mode from block {} with polling fallback", start_block);

        self.db
            .update_ingestion_state(start_block as i64, "live")
            .await?;

        info!("WebSocket subscription failed, using polling mode instead");
        self.run_polling_mode(start_block).await
    }

    async fn run_polling_mode(&self, mut last_block: u64) -> Result<()> {
        info!("Starting polling mode from block {}", last_block);
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let current_height = match self.blockchain_client.get_current_block_number().await {
                Ok(height) => height,
                Err(e) => {
                    error!("Failed to get current block number: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            if current_height > last_block {
                for block_num in (last_block + 1)..=current_height {
                    info!("Processing new block: {}", block_num);
                    
                    match self.blockchain_client.fetch_single_block_data(block_num).await {
                        Ok(block_data) => {
                            match self.process_and_store_blocks(vec![block_data]).await {
                                Ok(_) => {
                                    self.db
                                        .update_ingestion_state(block_num as i64, "live")
                                        .await?;
                                    debug!("Processed live block {}", block_num);
                                    last_block = block_num;
                                }
                                Err(e) => {
                                    error!("Failed to process block {}: {}", block_num, e);
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to fetch block {}: {}", block_num, e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn process_and_store_blocks(&self, block_data_batch: Vec<BlockData>) -> Result<()> {
        let mut blocks = Vec::new();
        let mut transactions = Vec::new();

        for block_data in block_data_batch {
            let block = Block {
                number: block_data.number as i64,
                timestamp: block_data.timestamp as i64,
                transaction_count: block_data.transactions.len() as i32,
                created_at: Utc::now(),
            };

            blocks.push(block);

            for (position, tx_hash) in block_data.transactions.iter().enumerate() {
                let transaction = Transaction {
                    hash: tx_hash.clone(),
                    block_number: block_data.number as i64,
                    position: position as i32,
                    created_at: Utc::now(),
                };
                transactions.push(transaction);
            }
        }

        self.db.batch_insert_blocks(&blocks).await?;
        self.db.batch_insert_transactions(&transactions).await?;

        debug!(
            "Stored {} blocks and {} transactions",
            blocks.len(),
            transactions.len()
        );

        Ok(())
    }
}