use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub mod repository;

use crate::models::{Block, IngestionState, Transaction};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn get_ingestion_state(&self) -> Result<IngestionState> {
        let state = sqlx::query_as::<_, IngestionState>(
            "SELECT id, last_processed_block, mode, updated_at FROM ingestion_state WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(state)
    }

    pub async fn update_ingestion_state(&self, last_block: i64, mode: &str) -> Result<()> {
        sqlx::query(
            "UPDATE ingestion_state SET last_processed_block = $1, mode = $2, updated_at = NOW() WHERE id = 1",
        )
        .bind(last_block)
        .bind(mode)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn batch_insert_blocks(&self, blocks: &[Block]) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO blocks (number, timestamp, transaction_count) "
        );

        query_builder.push_values(blocks, |mut b, block| {
            b.push_bind(block.number)
                .push_bind(block.timestamp)
                .push_bind(block.transaction_count);
        });

        query_builder.push(" ON CONFLICT (number) DO NOTHING");

        query_builder.build().execute(&self.pool).await?;

        Ok(())
    }

    pub async fn batch_insert_transactions(&self, transactions: &[Transaction]) -> Result<()> {
        if transactions.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO transactions (hash, block_number, position) "
        );

        query_builder.push_values(transactions, |mut b, tx| {
            b.push_bind(&tx.hash)
                .push_bind(tx.block_number)
                .push_bind(tx.position);
        });

        query_builder.push(" ON CONFLICT (hash) DO NOTHING");

        query_builder.build().execute(&self.pool).await?;

        Ok(())
    }
}