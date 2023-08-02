use super::{commons::IndexerError, transactions::Transaction};
use crate::{
    indexer::{event_parsers::get_event_parser, strategies::build_strategy},
    models::{Indexer, Network, Token, TokenTransfer, TokenType},
};
use log::{info, warn};
use postgres::Client;
use std::{env, str::FromStr, thread, time::Duration};
use tokio::runtime::Runtime;
use web3::{
    api::BaseFilter,
    transports::Http,
    types::{Address, FilterBuilder, Log, H256, U64},
    Web3,
};

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

fn cycle_body(
    client: &mut Client,
    indexer: &mut Indexer,
    runtime: &Runtime,
) -> Result<(), IndexerError> {
    let network = get_network(client, indexer)?;
    let transport = get_web3_transport(&network)?;
    let web3 = web3::Web3::new(transport);
    let range = get_blocks_range(
        get_block_number(&web3, runtime)?,
        indexer.last_block,
        network.max_step,
    );
    info!("Fetching events from {} to {} blocks", range.0, range.1);
    let strategy = build_strategy(indexer)?;
    info!(
        "Prepared strategy for fetching events: {}",
        indexer.strategy.as_str()
    );
    let tokens = get_tokens(client, indexer)?;
    info!(
        "Found {} tokens which are monitored by indexer",
        tokens.len()
    );
    let three_payload_topics = strategy.get_payload_topics(indexer.strategy_params.clone())?;
    for token in tokens.iter() {
        let events = token.token_type.get_events_hashes();
        info!(
            "Token {} has {} event type to handle",
            token.name.as_str(),
            events.len()
        );
        for event in events.iter() {
            let topics = [
                Some(vec![*event]),
                three_payload_topics[0].clone(),
                three_payload_topics[1].clone(),
                three_payload_topics[2].clone(),
            ];
            let filter = get_filter(&web3, runtime, token, range, &topics)?;
            let logs = get_logs(runtime, filter)?;
            let event_parser = get_event_parser(token);
            info!(
                "Fetched {} events for token {}",
                logs.len(),
                token.name.as_str()
            );
            let mut transactions = vec![];
            for log in logs.iter() {
                let transaction = event_parser.parse(log)?;
                info!("{transaction}");
                transactions.push(transaction);
            }
            info!("Saving to database {} token transfers", transactions.len());
            save_token_transfers(client, indexer, token, transactions)?;
        }
    }
    info!("Move indexer to block {}", range.1);
    update_last_block(client, indexer, range.1)?;
    Ok(())
}

fn get_network(client: &mut Client, indexer: &mut Indexer) -> Result<Network, IndexerError> {
    match Network::load_from_db(client, indexer.network_id) {
        Ok(network) => {
            info!("Network initialized {}", network.name.as_str());
            Ok(network)
        }
        Err(e) => Err(IndexerError {
            reason: format!("When fetching network occurred: {}", e.reason),
        }),
    }
}

fn get_web3_transport(network: &Network) -> Result<Http, IndexerError> {
    match web3::transports::Http::new(network.rpc_url.as_str()) {
        Ok(transport) => Ok(transport),
        Err(e) => Err(IndexerError {
            reason: e.to_string(),
        }),
    }
}

fn get_block_number(web3: &Web3<Http>, runtime: &Runtime) -> Result<u64, IndexerError> {
    match runtime.block_on(web3.eth().block_number()) {
        Ok(number) => Ok(number.as_u64()),
        Err(e) => Err(IndexerError {
            reason: e.to_string(),
        }),
    }
}

fn get_blocks_range(latest_block_blockchain: u64, latest_block_db: u64, step: u64) -> (u64, u64) {
    return (
        latest_block_db,
        std::cmp::min(latest_block_blockchain, latest_block_db + step),
    );
}

fn get_filter(
    web3: &Web3<Http>,
    runtime: &Runtime,
    token: &Token,
    block_range: (u64, u64),
    topics: &[Option<Vec<H256>>; 4],
) -> Result<BaseFilter<Http, Log>, IndexerError> {
    let filter_config = FilterBuilder::default()
        .address(vec![Address::from_str(token.address.as_str()).unwrap()])
        .from_block(web3::types::BlockNumber::Number(U64::from(block_range.0)))
        .to_block(web3::types::BlockNumber::Number(U64::from(block_range.1)))
        .topics(
            topics[0].clone(),
            topics[1].clone(),
            topics[2].clone(),
            topics[3].clone(),
        )
        .build();
    match runtime.block_on(web3.eth_filter().create_logs_filter(filter_config)) {
        Ok(base_filter) => Ok(base_filter),
        Err(e) => Err(IndexerError {
            reason: format!("During establishing new filter {e} occurred"),
        }),
    }
}

fn get_tokens(client: &mut Client, indexer: &Indexer) -> Result<Vec<Token>, IndexerError> {
    match Token::load_tokens_from_db_by_indexer(client, indexer) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(IndexerError {
            reason: format!("During fetching tokens {} occurred", e.reason),
        }),
    }
}

fn get_logs(runtime: &Runtime, filter: BaseFilter<Http, Log>) -> Result<Vec<Log>, IndexerError> {
    match runtime.block_on(filter.logs()) {
        Ok(logs) => Ok(logs),
        Err(e) => Err(IndexerError {
            reason: format!("Error occurred on logs fetching {e}"),
        }),
    }
}

fn save_token_transfers(
    client: &mut Client,
    indexer: &Indexer,
    token: &Token,
    transactions: Vec<Transaction>,
) -> Result<(), IndexerError> {
    match TokenTransfer::save_many(client, transactions, token, indexer) {
        Ok(()) => Ok(()),
        Err(e) => Err(IndexerError {
            reason: format!("Error occurred on token transfer saving: {}", e.reason),
        }),
    }
}

fn update_last_block(
    client: &mut Client,
    indexer: &mut Indexer,
    last_block: u64,
) -> Result<(), IndexerError> {
    match indexer.update_last_block(client, last_block) {
        Ok(()) => Ok(()),
        Err(e) => Err(IndexerError {
            reason: format!(
                "During updating last block of indexer {} to {} occurred error {}",
                indexer.name.as_str(),
                last_block,
                e.reason
            ),
        }),
    }
}
