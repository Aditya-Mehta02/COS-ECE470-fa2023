use crate::types::block::Block;
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;

pub struct Blockchain {
    blocks: HashMap<H256, Block>,
    tip: H256,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let genesis_block:Block = Block::new(H256::default()); // Create a genesis block (you can define it).
        let genesis_hash = genesis_block.hash();
        let mut blocks = HashMap::new();
        blocks.insert(genesis_hash, genesis_block.clone()); // Insert the genesis block into the HashMap.
        Self {
            blocks,
            tip: genesis_hash, // Set the tip to the hash of the genesis block.
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let cloned_block = block.clone();
        self.blocks.insert(block_hash, cloned_block); // Insert the block into the HashMap.
        // Update the tip if the inserted block's chain is longer.
        if block.get_parent() == self.tip {
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
            current_hash = block.get_parent(); // Follow the parent hash to the previous block.
        }
        longest_chain.reverse(); // Reverse the order to start from the genesis block.
        longest_chain
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