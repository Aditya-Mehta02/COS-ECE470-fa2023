use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use hex_literal::hex;
use std::collections::HashMap;

pub struct Blockchain {
    blocks: HashMap<H256, Block>,
    tip: H256,
    lengths: HashMap<H256, u32>,
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
        }
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

    /// Check if the blockchain contains a block with the given hash
    pub fn contains_block(&self, block_hash: &H256) -> bool {
        self.blocks.contains_key(block_hash)
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
