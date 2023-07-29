use super::commons::CycleError;
use crate::models::{Indexer, Network};
use postgres::Client;
use tokio::runtime::Runtime;
use web3::{transports::Http, types::{BlockId, Res}, Web3};
use std::{env, thread, time::Duration, future, collections::btree_map::Range};
use log::{warn, info};

pub fn start(client: &mut Client) {
    match env::var("INDEXER_NAME") {
        Ok(indexer_name) => {
            initialize_indexer(client, indexer_name);
        }
        Err(e) => {
            warn!("INDEXER_NAME: {}", e.to_string());
        }
    }
}

fn initialize_indexer(client: &mut Client, indexer_name: String) {
    match Indexer::load_from_db(client, &indexer_name) {
        Ok(mut indexer) => {
            info!("Starting indexer {indexer}");
            indexer_cycle(client, &mut indexer);
        }
        Err(e) => {
            warn!("On instantiating indexer {} occurred", e.reason);
        }
    }
}

fn indexer_cycle(client: &mut Client, indexer: &mut Indexer) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    loop {
        match cycle_body(client, indexer, &runtime) {
            Ok(_) => {
                thread::sleep(Duration::from_secs(indexer.short_sleep_seconds as u64));
            }
            Err(e) => {
                warn!("In cycle occurred error: {}", e.reason);
                break;
            }
        }
    }
}

fn cycle_body(client: &mut Client, indexer: &mut Indexer, runtime: &Runtime) -> Result<(), CycleError> {
    let network = get_network(client, indexer)?;
    let transport = get_web3_transport(&network)?;
    let web3 = web3::Web3::new(transport);
    let range = get_blocks_range(get_block_number(&web3, runtime)?, indexer.last_block, network.max_step);
    info!("Fetching events from {} to {} blocks", range.0, range.1);
    Ok(())
}

fn get_network(client: &mut Client, indexer: &mut Indexer) -> Result<Network, CycleError> {
    match Network::load_from_db(client, indexer.network_id) {
        Ok(network) => {
            info!("Network initialized");
            Ok(network)
        }
        Err(e) => Err(CycleError {
            reason: format!("When fetching network occurred: {}", e.reason),
        }),
    }
}

fn get_web3_transport(network: &Network) -> Result<Http, CycleError> {
    match web3::transports::Http::new(network.rpc_url.as_str()) {
        Ok(transport) => {
            Ok(transport)
        },
        Err(e) => {
            Err(CycleError { reason: e.to_string()})
        }
    }
}

fn get_block_number(web3: &Web3<Http>, runtime: &Runtime) -> Result<u64, CycleError> {
    match runtime.block_on(web3.eth().block_number()) {
        Ok(number) => {
            Ok(number.as_u64())
        },
        Err(e) => {
            Err(CycleError { reason: e.to_string() })
        }
    }
}

fn get_blocks_range(latest_block_blockchain: u64, latest_block_db: u64, step: u64) -> (u64, u64) {
    return (latest_block_db, std::cmp::min(latest_block_blockchain, latest_block_db + step))
}
