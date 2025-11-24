use std::collections::VecDeque;
use crate::models::{Block, Transaction};

pub struct BatchBuffer {
    blocks: VecDeque<Block>,
    transactions: VecDeque<Transaction>,
    max_size: usize,
}

impl BatchBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            blocks: VecDeque::with_capacity(max_size),
            transactions: VecDeque::with_capacity(max_size * 10),
            max_size,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push_back(block);
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push_back(transaction);
    }

    pub fn is_full(&self) -> bool {
        self.blocks.len() >= self.max_size
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty() && self.transactions.is_empty()
    }

    pub fn drain_blocks(&mut self) -> Vec<Block> {
        self.blocks.drain(..).collect()
    }

    pub fn drain_transactions(&mut self) -> Vec<Transaction> {
        self.transactions.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.transactions.clear();
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}