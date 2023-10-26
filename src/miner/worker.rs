use crate::blockchain::Blockchain;
use crate::network::message::Message;
// Import the Blockchain type
use crate::network::server::Handle as ServerHandle;
use crate::types::block::Block;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>, // Add the blockchain field
    net_server: ServerHandle,           // Handle to network's server
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>, // Add blockchain as an argument\
        net_server: &ServerHandle,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain), // Assign the blockchain to the field
            net_server: net_server.clone(),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let new_block = self
                .finished_block_chan
                .recv()
                .expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            self.blockchain.lock().unwrap().insert(&new_block);
            self.net_server
                .broadcast(Message::Blocks(vec![new_block.clone()]));
        }
    }
}
