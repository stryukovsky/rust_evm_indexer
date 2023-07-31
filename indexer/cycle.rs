use super::commons::CycleError;
use crate::{models::{Indexer, Network, Token, TokenType}, indexer::{strategies::build_strategy, event_parsers::get_event_parser}};
use postgres::Client;
use tokio::runtime::Runtime;
use web3::{transports::Http, types::{FilterBuilder, Address, U64, H256, Log}, Web3, api::BaseFilter};
use std::{env, thread, time::Duration, str::FromStr};
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
                thread::sleep(Duration::from_secs(indexer.long_sleep_seconds as u64));
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
    let strategy = build_strategy(indexer)?;
    info!("Prepared strategy for fetching events: {}", indexer.strategy.as_str());
    let tokens = get_tokens(client, indexer)?;
    info!("Found {} tokens which are monitored by indexer", tokens.len());
    let three_payload_topics = strategy.get_payload_topics(indexer.strategy_params.clone())?;
    for token in tokens.iter() {
        let token_type = TokenType::from(&token.token_type);
        let events = token_type.get_events_hashes();
        info!("Token {} has {} event type to handle", token.name.as_str(), events.len());
        for event in events.iter() {
            let topics = [Some(vec![*event]), three_payload_topics[0].clone(), three_payload_topics[1].clone(), three_payload_topics[2].clone()];
            let filter = get_filter(&web3, runtime, token, range, &topics)?;
            let logs = get_logs(runtime, filter)?;
            let event_parser = get_event_parser(token);
            info!("Fetched {} events for token {}", logs.len(), token.name.as_str());
            for log in logs.iter() {
                let transfer = event_parser.parse(log)?;
                println!("{transfer}")
            }
        }
    }
    info!("Move indexer to block {}", range.1);
    update_last_block(client, indexer, range.1)?;
    Ok(())
}

fn get_network(client: &mut Client, indexer: &mut Indexer) -> Result<Network, CycleError> {
    match Network::load_from_db(client, indexer.network_id) {
        Ok(network) => {
            info!("Network initialized {}", network.name.as_str());
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

fn get_filter(web3: &Web3<Http>, runtime: &Runtime, token: &Token, block_range: (u64, u64), topics:&[Option<Vec<H256>>; 4]) -> Result<BaseFilter<Http, Log>, CycleError> {
    let filter_config = FilterBuilder::default()
        .address(vec![Address::from_str(token.address.as_str()).unwrap()])
        .from_block(web3::types::BlockNumber::Number(U64::from(block_range.0)))
        .to_block(web3::types::BlockNumber::Number(U64::from(block_range.1)))
        .topics(topics[0].clone(), topics[1].clone(), topics[2].clone(), topics[3].clone()).build();
    match runtime.block_on(web3.eth_filter().create_logs_filter(filter_config)) {
        Ok(base_filter) => {
            Ok(base_filter)
        },
        Err(e) => {
            Err(CycleError { reason: format!("During establishing new filter {e} occurred") })
        }
    }
}  

fn get_tokens(client: &mut Client, indexer: &Indexer) -> Result<Vec<Token>, CycleError> {
    match Token::load_tokens_from_db_by_indexer(client, indexer) {
        Ok(tokens) => {
            Ok(tokens)
        },
        Err(e) => {
            Err(CycleError { reason: format!("During fetching tokens {} occurred", e.reason) })
        }
    }
}


fn get_logs(runtime: &Runtime, filter: BaseFilter<Http, Log>) -> Result<Vec<Log>, CycleError> {
    match runtime.block_on(filter.logs()) {
        Ok(logs) => {
            Ok(logs)
        },
        Err(e) => {
            Err(CycleError { reason: format!("Error occurred on logs fetching {e}") })
        }
    }
}

fn update_last_block(client: &mut Client, indexer: &mut Indexer, last_block: u64)-> Result<(), CycleError>{
    match indexer.update_last_block(client, last_block) {
        Ok(()) => {
            Ok(())
        },
        Err(e) => {
            Err(CycleError { reason: format!("During updating last block of indexer {} to {} occurred error {}", indexer.name.as_str(), last_block, e.reason) })
        }
    }
}
