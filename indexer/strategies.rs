use serde_json::Value;
use std::vec;
use web3::types::H256;

use crate::models::{Indexer, IndexerStrategy};

use super::{byte_parsers::hex_string_to_bytes32, commons::CycleError};

pub trait Strategy {
    fn get_payload_topics(
        &self,
        strategy_params: Option<Value>,
    ) -> Result<[Option<Vec<H256>>; 3], CycleError>;

    fn get_strategy_params(&self, strategy_params: Option<Value>) -> Result<Value, CycleError> {
        match strategy_params {
            Some(strategy_json) => Ok(strategy_json),
            None => Err(CycleError {
                reason: String::from(
                    "Expected some strategy JSON value with `recipient` in it, found no JSON",
                ),
            }),
        }
    }
}

struct RecipientStrategy();

impl Strategy for RecipientStrategy {
    fn get_payload_topics(
        &self,
        strategy_params: Option<Value>,
    ) -> Result<[Option<Vec<H256>>; 3], CycleError> {
        let strategy_json = self.get_strategy_params(strategy_params)?;
        let recipient = Self::get_recipient(strategy_json)?;
        let recipient_hex = hex_string_to_bytes32(&recipient)?;
        Ok([None, Some(vec![recipient_hex]), None])
    }
}

impl RecipientStrategy {
    fn get_recipient(strategy_json: Value) -> Result<String, CycleError> {
        if let Some(recipient) = strategy_json.get("recipient") {
            if recipient.is_string() {
                let mut recipient_address = recipient.to_string();
                if recipient_address.starts_with('"') {
                    assert_eq!(recipient_address.remove(0), '"')
                }
                if recipient_address.ends_with('"') {
                    assert_eq!(recipient_address.remove(recipient_address.len() - 1), '"')
                }
                Ok(recipient_address)
            } else {
                Err(CycleError {
                    reason: String::from("Expected recipient value to be string"),
                })
            }
        } else {
            Err(CycleError {
                reason: String::from(
                    "Expected recipient key containing recipient address not found",
                ),
            })
        }
    }
}

struct TokenScanStrategy();

impl Strategy for TokenScanStrategy {
    fn get_payload_topics(&self, _: Option<Value>) -> Result<[Option<Vec<H256>>; 3], CycleError> {
        Ok([None, None, None])
    }
}

pub fn build_strategy(indexer: &Indexer) -> Result<Box<dyn Strategy>, CycleError> {
    match IndexerStrategy::from(&indexer.strategy) {
        IndexerStrategy::Recipient => Ok(Box::new(RecipientStrategy {})),
        IndexerStrategy::Sender => todo!(),
        IndexerStrategy::TokenScan => Ok(Box::new(TokenScanStrategy {})),
    }
}
