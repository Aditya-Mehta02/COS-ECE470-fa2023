use crate::types::hash::{Hashable, H256};
use crate::types::transaction::SignedTransaction;
use hex_literal::hex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    transactions: Vec<SignedTransaction>,
}

impl Content {
    pub fn new() -> Self {
        let mut transactions = Vec::new();
        Content { transactions }
    }
    pub fn add_transactions(&mut self, transactions: Vec<SignedTransaction>) {
        self.transactions.extend(transactions);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: u128,
    merkle_root: H256,
}

impl Header {
    pub fn new(parent: H256, nonce: u32) -> Self {
        let mut difficulty =
            hex!("000010ffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        let mut timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut merkle_root = H256::from([0; 32]);
        Header {
            parent: parent,
            nonce: nonce,
            difficulty,
            timestamp,
            merkle_root,
        }
    }

    pub fn get_genesis_header() -> Self {
        let parent = H256::from([0; 32]); // Genesis block has no parent
        let nonce = 0u32; // An arbitrary fixed nonce for genesis
        let difficulty =
            hex!("000010ffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        // Fixed timestamp for genesis block, for example, the UNIX timestamp of a specific memorable date
        let timestamp = 1615523200000; // This is a sample timestamp for 2021-03-12 00:00:00
        let merkle_root = H256::from([0; 32]); // Genesis block's merkle root could be all zeros

        Header {
            parent,
            nonce,
            difficulty,
            timestamp,
            merkle_root,
        }
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let encoded = bincode::serialize(&self).expect("failed to serialize");
        ring::digest::digest(&ring::digest::SHA256, &encoded).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    content: Content,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_genesis_block() -> Self {
        let genesis_parent: H256 =
            hex!("00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        let genesis_nonce = 0u32; // or some predetermined value
        let genesis_header = Header::get_genesis_header();
        let genesis_content = Content::new();
        Block {
            header: genesis_header,
            content: genesis_content,
        }
    }

    pub fn new(parent: H256) -> Self {
        let mut rng = rand::thread_rng();
        let mut nonce = rng.gen::<u32>();
        let mut header = Header::new(parent, nonce);
        let mut content = Content::new();
        Block { header, content }
    }

    // Setter method for changing the nonce
    pub fn set_nonce(&mut self, new_nonce: u32) {
        self.header.nonce = new_nonce;
    }

    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }

    // Method to get a reference to the transactions within the block
    pub fn get_transactions(&self) -> &Vec<SignedTransaction> {
        &self.content.transactions
    }

    // Optionally, if you need to modify the transactions, add this method
    pub fn get_transactions_mut(&mut self) -> &mut Vec<SignedTransaction> {
        &mut self.content.transactions
    }

    pub fn get_content_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let mut block = Block::new(*parent);
    block
}
