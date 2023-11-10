use crate::blockchain::Blockchain;
use crate::types::hash::H256;
use crate::types::transaction::SignedTransaction;
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; // Import the Blockchain type

use super::hash::Hashable;
use super::transaction::verify;

pub struct Mempool {
    transactions: HashMap<H256, SignedTransaction>,
    // other fields as necessary
}

impl Mempool {
    /// Create a new mempool
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            // initialize other fields
        }
    }

    /// Add a transaction to the mempool if it is valid
    pub fn add_transaction(&mut self, tx: SignedTransaction) {
        let tx_hash = tx.hash(); // Assume SignedTransaction implements the Hashable trait
        if self.is_valid(&tx) && !self.transactions.contains_key(&tx_hash) {
            self.transactions.insert(tx_hash, tx);
        }
    }

    /// Checks if a transaction is valid
    pub fn is_valid(&self, tx: &SignedTransaction) -> bool {
        // Implement validity checks here
        verify(tx.transaction(), tx.public_key(), tx.signature())
    }

    /// Remove transactions that are included in a block
    pub fn remove_transactions(&mut self, block_transactions: &[H256]) {
        for tx_hash in block_transactions {
            self.transactions.remove(tx_hash);
        }
    }

    /// Method to get transactions for mining a new block
    /// Here you could implement logic to choose transactions based on fees or other criteria
    pub fn get_transactions_for_block(
        &self,
        max_size: usize,
        blockchain: &Blockchain, // Add a reference to the blockchain
    ) -> Vec<SignedTransaction> {
        let mut block_transactions = Vec::new();

        for tx in self.transactions.values() {
            if block_transactions.len() >= max_size {
                break;
            }
            // Check if the transaction is already included in the blockchain.
            let tx_hash = tx.hash();
            if !blockchain.contains_transaction(&tx_hash) {
                block_transactions.push(tx.clone());
            }
        }

        block_transactions
    }

    pub fn contains_transaction(&self, tx_hash: &H256) -> bool {
        self.transactions.contains_key(tx_hash)
    }

    /// Retrieve a transaction from the mempool by its hash
    pub fn get_transaction(&self, tx_hash: &H256) -> Option<&SignedTransaction> {
        self.transactions.get(tx_hash)
    }
}

// Shared mempool type definition
pub type SharedMempool = Arc<Mutex<Mempool>>;

// Usage in miner or network worker
// let mempool: SharedMempool = Arc::new(Mutex::new(Mempool::new()));
// Now you can pass `mempool` to the miner and network worker
