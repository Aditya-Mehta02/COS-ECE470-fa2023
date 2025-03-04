#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod api;
pub mod blockchain;
pub mod generator;
pub mod miner;
pub mod network;
pub mod types;

use api::Server as ApiServer;
use blockchain::Blockchain;
use clap::clap_app;
use log::{error, info};
use ring::signature;
use smol::channel;
use std::net;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use types::mempool::Mempool;

extern crate ring;
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::fs::File;
use std::io::Write;
extern crate base64;

fn generate_and_save_keys() -> Result<(), std::io::Error> {
    let rng = ring::rand::SystemRandom::new();

    // Generate a key pair
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let pkcs8_bytes: &[u8] = pkcs8_bytes.as_ref();

    // Save private key to file
    let mut file = File::create("key_pair.pem")?;
    file.write_all(pkcs8_bytes)?;

    Ok(())
}

fn main() {
    if let Err(e) = generate_and_save_keys() {
        println!("Failed to generate keys: {}", e);
    }
    // parse command line arguments
    let matches = clap_app!(Bitcoin =>
     (version: "0.1")
     (about: "Bitcoin client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to at start")
     (@arg p2p_workers: --("p2p-workers") [INT] default_value("4") "Sets the number of worker threads for P2P server")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);

    // start the p2p server
    let (server_ctx, server) = network::server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();

    // start the worker
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });

    // Initialize the mempool
    let mempool = Mempool::new();
    let mempool = Arc::new(Mutex::new(mempool));
    let cloned_blockchain = Arc::clone(&blockchain);
    let cloned_mempool = Arc::clone(&mempool); // Clone the Arc to pass to the worker
    let worker_ctx = network::worker::Worker::new(
        p2p_workers,
        msg_rx,
        &server,
        cloned_blockchain,
        cloned_mempool,
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner, finished_block_chan) =
        miner::new(&Arc::clone(&blockchain), &Arc::clone(&mempool));
    let miner_worker_ctx = miner::worker::Worker::new(
        &server,
        finished_block_chan,
        &Arc::clone(&blockchain),
        &server,
    );
    miner_ctx.start();
    miner_worker_ctx.start();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }

    // start the API server
    ApiServer::start(api_addr, &miner, &server, &blockchain, &mempool);

    loop {
        std::thread::park();
    }
}
