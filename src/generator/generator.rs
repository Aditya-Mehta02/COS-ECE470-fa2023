use log::info;
use std::thread;
use std::time;

use crate::network::message::Message;
use crate::network::server::Handle as NetworkServerHandle;
use crate::types::hash::Hashable;
use crate::types::mempool::Mempool;
use crate::types::transaction::{SignedTransaction, Transaction};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TransactionGenerator {}

impl TransactionGenerator {
    pub fn new() -> Self {
        Self {}
    }

    // This function needs access to the network server handle and the mempool
    pub fn start(self, theta: u64, network: NetworkServerHandle, mempool: Arc<Mutex<Mempool>>) {
        thread::Builder::new()
            .name("transaction-generator".to_string())
            .spawn(move || {
                self.generate_transactions(theta, network, mempool);
            })
            .unwrap();
        info!("Transaction generator started");
    }

    fn generate_transactions(
        &self,
        theta: u64,
        network: NetworkServerHandle,
        mempool: Arc<Mutex<Mempool>>,
    ) {
        loop {
            let signed_transaction = SignedTransaction::get_random_signed_transaction();
            // Lock the mutex to get access to the mempool.
            let mut mempool_guard = mempool.lock().unwrap();
            // Now you can add the transaction to the mempool.
            mempool_guard.add_transaction(signed_transaction.clone());
            drop(mempool_guard); // Explicitly drop the lock if you want to release it here

            network.broadcast(Message::NewTransactionHashes(vec![
                signed_transaction.hash()
            ]));

            if theta != 0 {
                let interval = time::Duration::from_millis(10 * theta);
                thread::sleep(interval);
            }
        }
    }
}
