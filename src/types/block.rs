use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};
use crate::types::transaction::{SignedTransaction};
use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    transactions: Vec<SignedTransaction>,
}

impl Content {
    pub fn new() -> Self {
        let mut transactions = Vec::new();
        Content {
            transactions
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: std::time::SystemTime,
    merkle_root: H256,
}

impl Header {
    pub fn new(parent: H256, nonce: u32) -> Self {
        let mut difficulty = H256::from([0; 32]);
        let mut timestamp = std::time::SystemTime::now();
        let mut merkle_root = H256::from([0; 32]);
        Header {
            parent: parent,
            nonce: nonce,
            difficulty,
            timestamp,
            merkle_root
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
    pub fn new(parent: H256) -> Self {
        let mut rng = rand::thread_rng();
        let mut nonce = rng.gen::<u32>();
        let mut header = Header::new(parent, nonce);
        let mut content = Content::new();
        Block {
            header,
            content,
        }
    }

    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let mut block = Block::new(*parent);
    block
}