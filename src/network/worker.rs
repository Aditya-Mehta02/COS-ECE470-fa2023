use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use crate::types::mempool::{self, Mempool};
use crate::types::transaction::{SignedTransaction, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; // Import the Blockchain type // Add for orphan block buffer // Assuming you have a Mempool struct defined

use log::{debug, error, warn};

use std::thread;

#[cfg(any(test, test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test, test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]

pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>, // Add the blockchain field
    orphan_blocks: HashMap<H256, Block>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: Arc<Mutex<Blockchain>>, // Add blockchain as an argument
        mempool: Arc<Mutex<Mempool>>,       // Add mempool as an argument
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: blockchain, // Assign the blockchain to the field
            orphan_blocks: HashMap::new(),
            mempool: mempool,
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let mut cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn process_block(&mut self, block: &Block) -> bool {
        // PoW check
        if block.hash() > block.get_difficulty() {
            warn!("Block's hash does not satisfy PoW requirement.");
            return false;
        }

        let mut blockchain = self.blockchain.lock().unwrap();

        // Check if the difficulty is as expected
        let parent_difficulty = if !blockchain.contains_block(&block.get_parent()) {
            block.get_difficulty()
        } else {
            match blockchain.get_block(&block.get_parent()) {
                Some(parent_block) => parent_block.get_difficulty(),
                None => return false,
            }
        };
        if block.get_difficulty() != parent_difficulty {
            warn!("Block's difficulty doesn't match with the parent's difficulty.");
            return false;
        }

        // Check if the block's parent exists
        if !blockchain.contains_block(&block.get_parent()) {
            // Add to orphan buffer
            self.orphan_blocks.insert(block.get_parent(), block.clone());
            // Send GetBlocks message with this parent hash
            println!(
                "send GetBlocks msg with parent hash: {}, in process_block()",
                block.get_parent()
            );
            self.server
                .broadcast(Message::GetBlocks(vec![block.get_parent()]));
            return false;
        }

        // If all checks passed, add block to the blockchain
        println!(
            "adding block: {} to blockchain, in process_block()",
            block.hash()
        );

        blockchain.insert(&block);
        true
    }

    fn process_orphan_blocks(&mut self, parent_hash: H256) {
        let mut blockchain = self.blockchain.lock().unwrap();

        // Get orphan blocks associated with the parent_hash
        let mut orphan_block = self.orphan_blocks.remove(&parent_hash);

        while let Some(block) = orphan_block {
            // Add the block to the blockchain
            println!("adding block: {} to blockchain", block.hash());
            blockchain.insert(&block);
            // Get the next orphan block
            orphan_block = self.orphan_blocks.remove(&block.hash());
        }
    }

    fn worker_loop(&mut self) {
        print!("worker started");
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hashes) => {
                    println!("receiving NewBlockHashes msg");
                    let blockchain = self.blockchain.lock().unwrap();
                    let unknown_hashes: Vec<H256> = hashes
                        .into_iter()
                        .filter(|hash| !blockchain.contains_block(hash))
                        .collect();
                    if !unknown_hashes.is_empty() {
                        peer.write(Message::GetBlocks(unknown_hashes));
                    }
                }
                Message::GetBlocks(hashes) => {
                    println!("receiving GetBlocks msg");
                    let blockchain = self.blockchain.lock().unwrap();
                    let blocks: Vec<Block> = hashes
                        .iter()
                        .filter_map(|hash: &H256| blockchain.get_block(hash).cloned())
                        .collect();
                    if !blocks.is_empty() {
                        peer.write(Message::Blocks(blocks));
                    }
                }
                Message::Blocks(blocks) => {
                    println!("receiving Blocks msg");
                    let mut new_hashes = Vec::new();
                    for block in blocks {
                        println!("{}", block.hash());
                        if !self.process_block(&block) {
                            continue;
                        }
                        println!("adding new chain");
                        new_hashes.push(block.hash());
                        self.process_orphan_blocks(block.hash());
                    }
                    if !new_hashes.is_empty() {
                        println!("broadcasting NewBlockHashes");
                        self.server.broadcast(Message::NewBlockHashes(new_hashes));
                    }
                }

                Message::NewTransactionHashes(tx_hashes) => {
                    println!("Receiving NewTransactionHashes msg");
                    let blockchain = self.blockchain.lock().unwrap();
                    let mempool = self.mempool.lock().unwrap();

                    let unknown_hashes: Vec<H256> = tx_hashes
                        .into_iter()
                        .filter(|hash| {
                            !blockchain.contains_transaction(hash)
                                && !mempool.contains_transaction(hash)
                        })
                        .collect();

                    if !unknown_hashes.is_empty() {
                        peer.write(Message::GetTransactions(unknown_hashes));
                    }
                }
                Message::GetTransactions(tx_hashes) => {
                    println!("Receiving GetTransactions msg");
                    let mempool = self.mempool.lock().unwrap();

                    let transactions: Vec<SignedTransaction> = tx_hashes
                        .iter()
                        .filter_map(|hash| mempool.get_transaction(hash))
                        .cloned()
                        .collect();

                    if !transactions.is_empty() {
                        peer.write(Message::Transactions(transactions));
                    }
                }
                Message::Transactions(transactions) => {
                    println!("Receiving Transactions msg");
                    let mut mempool = self.mempool.lock().unwrap();

                    for tx in transactions {
                        if !mempool.contains_transaction(&tx.hash()) && mempool.is_valid(&tx) {
                            // `verify_signature` is a new method to be implemented in SignedTransaction
                            mempool.add_transaction(tx);
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(any(test, test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>,
}
#[cfg(any(test, test_utilities))]
impl TestMsgSender {
    fn new() -> (
        TestMsgSender,
        smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    ) {
        let (s, r) = smol::channel::unbounded();
        (TestMsgSender { s }, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test, test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    // Initialize the mempool
    let mempool = Mempool::new();
    let shared_mempool = Arc::new(Mutex::new(mempool));
    let blockchain = Blockchain::new();
    let block_hashes = blockchain.all_blocks_in_longest_chain(); // Assuming this method exists based on description.
    let worker = Worker::new(
        1,
        msg_chan,
        &server,
        Arc::new(Mutex::new(blockchain)),
        Arc::clone(&shared_mempool),
    );
    worker.start();
    (test_msg_sender, server_receiver, block_hashes)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;
    use ntest::timeout;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver =
            test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
