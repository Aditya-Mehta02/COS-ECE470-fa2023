use crate::blockchain::Blockchain;
use crate::generator::generator::TransactionGenerator;
use crate::miner::Handle as MinerHandle;
use crate::network::message::Message;
use crate::network::server::Handle as NetworkServerHandle;
use crate::types::hash::Hashable;
use crate::types::mempool::{self, Mempool};
use crate::types::state::State;
use serde::Serialize;

use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{clone, thread};
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;

pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
    network: NetworkServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        network: &NetworkServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            network: network.clone(),
            blockchain: Arc::clone(blockchain),
            mempool: Arc::clone(mempool),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let network = server.network.clone();
                let blockchain = Arc::clone(&server.blockchain);
                let mempool = Arc::clone(&server.mempool);
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/tx-generator/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let theta = match params.get("theta") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing theta");
                                    return;
                                }
                            };
                            let theta = match theta.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing theta: {}", e)
                                    );
                                    return;
                                }
                            };
                            let tx_generator = TransactionGenerator::new();
                            tx_generator.start(theta, network, mempool);
                            respond_result!(req, true, "Transaction generator started");
                            // unimplemented!()
                            // respond_result!(req, false, "unimplemented!");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        "/blockchain/longest-chain" => {
                            let blockchain = blockchain.lock().unwrap();
                            let v = blockchain.all_blocks_in_longest_chain();
                            let v_string: Vec<String> =
                                v.into_iter().map(|h| h.to_string()).collect();
                            respond_json!(req, v_string);
                        }
                        "/blockchain/longest-chain-tx" => {
                            let blockchain = blockchain.lock().unwrap();
                            let longest_chain_hashes = blockchain.all_blocks_in_longest_chain();
                            let mut tx_hashes: Vec<Vec<String>> = Vec::new();

                            for block_hash in longest_chain_hashes {
                                if let Some(block) = blockchain.get_block(&block_hash) {
                                    let block_tx_hashes: Vec<String> = block
                                        .get_transactions()
                                        .iter()
                                        .map(|tx| format!("{}", tx.hash()))
                                        .collect();
                                    tx_hashes.push(block_tx_hashes);
                                }
                            }

                            // Serialize the transaction hashes to JSON
                            // let tx_hashes_json = serde_json::to_string(&tx_hashes).unwrap();
                            respond_json!(req, tx_hashes);

                            // unimplemented!()
                            // respond_result!(req, false, "unimplemented!");
                        }
                        "/blockchain/longest-chain-tx-count" => {
                            let blockchain = blockchain.lock().unwrap();
                            let longest_chain_hashes = blockchain.all_blocks_in_longest_chain();
                            let tx_count: usize = longest_chain_hashes
                                .iter()
                                .map(|block_hash| {
                                    blockchain
                                        .get_block(block_hash)
                                        .map(|block| block.get_transactions().len())
                                        .unwrap_or(0)
                                })
                                .sum();

                            respond_json!(req, tx_count);

                            // unimplemented!()
                            // respond_result!(req, false, "unimplemented!");
                        }
                        "/blockchain/state" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let block_str = match params.get("block") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing block number");
                                    return;
                                }
                            };
                            let block_number = match block_str.parse::<u32>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing block number: {}", e)
                                    );
                                    return;
                                }
                            };
                            let blockchain = blockchain.lock().unwrap();
                            match blockchain.get_state_up_to_block(block_number) {
                                Ok(state) => {
                                    let mut accounts: Vec<(String, u64, u128)> = state
                                        .get_accounts()
                                        .iter()
                                        .map(|(address, info)| {
                                            (
                                                address.to_string(),
                                                info.get_nonce(),
                                                info.get_balance(),
                                            )
                                        })
                                        .collect();

                                    accounts.sort_by(|a, b| a.0.cmp(&b.0));

                                    let accounts_str: Vec<String> = accounts
                                        .iter()
                                        .map(|(address, nonce, balance)| {
                                            format!("({}, {}, {})", address, nonce, balance)
                                        })
                                        .collect();

                                    respond_json!(req, accounts_str);
                                }
                                Err(e) => respond_result!(req, false, e),
                            }
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
