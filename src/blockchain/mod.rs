use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use crate::types::state::{self, State};
use hex_literal::hex;
use std::collections::HashMap;
use std::thread::current;

pub struct Blockchain {
    blocks: HashMap<H256, Block>,
    tip: H256,
    lengths: HashMap<H256, u32>,
    state: State,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let genesis_block: Block = Block::get_genesis_block();
        let genesis_hash = genesis_block.hash();
        println!("genesis_hash: {}", genesis_hash);
        let mut blocks = HashMap::new();
        let mut lengths = HashMap::new();
        blocks.insert(genesis_hash, genesis_block.clone());
        lengths.insert(genesis_hash, 0);
        Self {
            blocks,
            tip: genesis_hash,
            lengths,
            state: State::new(),
        }
    }

    pub fn get_state_up_to_block(&self, mut block_number: u32) -> Result<State, String> {
        let mut state = State::new(); // Start with a new state
        let mut current_hash = self.tip;
        let mut current_block_number = self.lengths.get(&current_hash).copied().unwrap_or_default();
        if block_number > current_block_number {
            block_number = current_block_number;
        }

        while current_block_number >= block_number {
            while current_block_number > 0 && current_block_number <= block_number {
                if let Some(block) = self.blocks.get(&current_hash) {
                    for transaction in block.get_transactions() {
                        state.apply_transaction(transaction)?;
                    }
                    current_hash = block.get_parent();
                    current_block_number =
                        self.lengths.get(&current_hash).copied().unwrap_or_default();
                } else {
                    return Err("Block not found".to_string());
                }
            }
            if current_block_number > 0 {
                let block = self.blocks.get(&current_hash);
                current_hash = block.unwrap().get_parent();
                current_block_number = self.lengths.get(&current_hash).copied().unwrap_or_default();
            }
        }
        Ok(state)
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let cloned_block = block.clone();
        self.blocks.insert(block_hash, cloned_block);
        self.lengths.insert(
            block_hash,
            self.lengths.get(&block.get_parent()).unwrap_or(&0) + 1,
        );
        if self.lengths.get(&block_hash) > self.lengths.get(&self.tip) {
            self.tip = block_hash;
        }
        // Apply transactions to the state
        for transaction in block.get_transactions() {
            match self.state.apply_transaction(transaction) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to apply transaction: {}", e),
            }
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut current_hash = self.tip;
        let mut longest_chain = Vec::new();
        while let Some(block) = self.blocks.get(&current_hash) {
            longest_chain.push(current_hash);
            current_hash = block.get_parent();
        }
        longest_chain.reverse();
        longest_chain
    }

    /// Retrieve a block from the blockchain by its hash
    pub fn get_block(&self, block_hash: &H256) -> Option<&Block> {
        self.blocks.get(block_hash)
    }

    /// Retrieve blockchain state
    pub fn get_state(&self) -> &State {
        &self.state
    }

    /// Check if the blockchain contains a block with the given hash
    pub fn contains_block(&self, block_hash: &H256) -> bool {
        self.blocks.contains_key(block_hash)
    }

    /// Check if the blockchain contains a transaction with the given hash
    pub fn contains_transaction(&self, tx_hash: &H256) -> bool {
        // Iterate over all blocks and check each transaction
        for block in self.blocks.values() {
            for transaction in block.get_transactions() {
                if &transaction.hash() == tx_hash {
                    return true;
                }
            }
        }
        false
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
