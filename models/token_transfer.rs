use std::vec;

use postgres::Client;
use rust_decimal::Decimal;
use web3::types::U256;

use crate::indexer::transactions::Transaction;

const TABLE_NAME: &'static str = "indexer_api_tokentransfer";
const INSERT_QUERY: &'static str = "INSERT INTO indexer_api_tokentransfer (operator, sender, recipient, tx_hash, token_id, amount, token_instance_id, fetched_by_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

use crate::indexer::transactions::TransferredToken::{Fungible, ERC1155, NFT};

use super::{Indexer, ModelError, Token};

pub struct TokenTransfer {
    id: i64,
    operator: String,
    sender: String,
    recipient: String,
    tx_hash: String,
    token_id: String,
    amount: String,
    token_instance_id: i64,
    fetched_by_id: i64,
}

impl TokenTransfer {
    pub fn save_many(
        client: &mut Client,
        transactions: Vec<Transaction>,
        token: &Token,
        indexer: &Indexer,
    ) -> Result<(), ModelError> {
        let mut db_tx = Self::start_db_tx(client)?;
        for transaction in transactions.iter() {
            for token_transfer in Self::build_from_transaction(transaction, token, indexer).iter() {
                token_transfer.add_to_db_tx(&mut db_tx)?;
            }
        }
        match db_tx.commit() {
            Ok(_) => Ok(()),
            Err(e) => Err(ModelError {
                reason: format!("During commit transaction in database occurred {e}"),
            }),
        }
    }

    fn start_db_tx(client: &mut Client) -> Result<postgres::Transaction, ModelError> {
        match client.transaction() {
            Ok(db_tx) => Ok(db_tx),
            Err(e) => Err(ModelError {
                reason: format!("During preparing database transaction occurred {e}"),
            }),
        }
    }

    pub fn add_to_db_tx(&self, db_tx: &mut postgres::Transaction) -> Result<(), ModelError> {
        match db_tx.execute(
            format!("INSERT INTO {TABLE_NAME} (operator, sender, recipient, tx_hash, token_id, amount, token_instance_id, fetched_by_id) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", &self.sender, &self.sender, &self.recipient, &self.tx_hash, &self.token_id, &self.amount, &self.token_instance_id, &self.fetched_by_id).as_str(),
            &[],
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(ModelError {
                reason: format!(
                    "During preparing Transaction {} to be saved {e} occurred",
                    &self.tx_hash
                ),
            }),
        }
    }

    pub fn build_from_transaction(
        transaction: &Transaction,
        token: &Token,
        indexer: &Indexer,
    ) -> Vec<Self> {
        match &transaction.transferred_token {
            Fungible { address, amount } => {
                vec![Self {
                    id: 0,
                    operator: transaction.sender.clone(),
                    sender: transaction.sender.clone(),
                    recipient: transaction.recipient.clone(),
                    tx_hash: transaction.tx_hash.clone(),
                    token_id: String::from("0"),
                    amount: amount.to_string(),
                    token_instance_id: token.id,
                    fetched_by_id: indexer.id,
                }]
            }
            NFT { address, token_id } => {
                vec![Self {
                    id: 0,
                    operator: transaction.sender.clone(),
                    sender: transaction.sender.clone(),
                    recipient: transaction.recipient.clone(),
                    tx_hash: transaction.tx_hash.clone(),
                    token_id: token_id.to_string(),
                    amount: String::from("0"),
                    token_instance_id: token.id,
                    fetched_by_id: indexer.id,
                }]
            }
            ERC1155 {
                address,
                token_ids,
                amounts,
            } => {
                assert_eq!(token_ids.len(), amounts.len());
                let mut result = vec![];
                for (i, token_id) in token_ids.iter().enumerate() {
                    let amount = amounts.get(i).unwrap();
                    result.push(Self {
                        id: 0,
                        operator: transaction.sender.clone(),
                        sender: transaction.sender.clone(),
                        recipient: transaction.recipient.clone(),
                        tx_hash: transaction.tx_hash.clone(),
                        token_id: token_id.to_string(),
                        amount: amount.to_string(),
                        token_instance_id: token.id,
                        fetched_by_id: indexer.id,
                    })
                }
                result
            }
        }
    }
}
