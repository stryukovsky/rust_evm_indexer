use serde_json::Value;
use std::vec;
use web3::types::H256;

use crate::models::{Indexer, IndexerStrategy};

use super::{byte_parsers::hex_string_to_bytes32, commons::IndexerError};

fn get_strategy_params(strategy_params: Option<Value>) -> Result<Value, IndexerError> {
    match strategy_params {
        Some(strategy_json) => Ok(strategy_json),
        None => Err(IndexerError {
            reason: String::from(
                "Expected some strategy JSON value with `recipient` in it, found no JSON",
            ),
        }),
    }
}

fn get_address(strategy_json: Value, key: &'static str) -> Result<String, IndexerError> {
    if let Some(value) = strategy_json.get("recipient") {
        if value.is_string() {
            let mut address = value.to_string();
            if address.starts_with('"') {
                assert_eq!(address.remove(0), '"')
            }
            if address.ends_with('"') {
                assert_eq!(address.remove(address.len() - 1), '"')
            }
            Ok(address)
        } else {
            Err(IndexerError {
                reason: format!("Expected {key} value to be string"),
            })
        }
    } else {
        Err(IndexerError {
            reason: format!("Expected {key} key containing recipient address not found"),
        })
    }
}

pub trait Strategy {
    fn get_payload_topics(
        &self,
        strategy_params: Option<Value>,
    ) -> Result<[Option<Vec<H256>>; 3], IndexerError>;
}

struct RecipientStrategy();
const RECIPIENT_KEY: &'static str = "recipient";
impl Strategy for RecipientStrategy {
    fn get_payload_topics(
        &self,
        strategy_params: Option<Value>,
    ) -> Result<[Option<Vec<H256>>; 3], IndexerError> {
        let strategy_json = get_strategy_params(strategy_params)?;
        let recipient = get_address(strategy_json, RECIPIENT_KEY)?;
        let recipient_hex = hex_string_to_bytes32(&recipient)?;
        Ok([None, Some(vec![recipient_hex]), None])
    }
}

struct SenderStrategy();
const SENDER_KEY: &'static str = "sender";
impl Strategy for SenderStrategy {
    fn get_payload_topics(
        &self,
        strategy_params: Option<Value>,
    ) -> Result<[Option<Vec<H256>>; 3], IndexerError> {
        let strategy_json = get_strategy_params(strategy_params)?;
        let sender = get_address(strategy_json, SENDER_KEY)?;
        let sender_hex = hex_string_to_bytes32(&sender)?;
        Ok([None, Some(vec![sender_hex]), None])
    }
}

struct TokenScanStrategy();
impl Strategy for TokenScanStrategy {
    fn get_payload_topics(&self, _: Option<Value>) -> Result<[Option<Vec<H256>>; 3], IndexerError> {
        Ok([None, None, None])
    }
}

pub fn build_strategy(indexer: &Indexer) -> Result<Box<dyn Strategy>, IndexerError> {
    match IndexerStrategy::from(&indexer.strategy) {
        IndexerStrategy::Recipient => Ok(Box::new(RecipientStrategy {})),
        IndexerStrategy::Sender => Ok(Box::new(SenderStrategy {})),
        IndexerStrategy::TokenScan => Ok(Box::new(TokenScanStrategy {})),
    }
}
