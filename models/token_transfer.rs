use postgres::Client;
use rust_decimal::Decimal;

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
    token_id: Decimal,
    amount: Decimal,
    token_instance_id: i64,
    fetched_by_id: i64,
}

impl TokenTransfer {
    pub fn save_many(client: &mut Client, transactions: Vec<Transaction>, token: &Token, indexer: &Indexer) -> Result<(), ModelError> {
        let mut db_tx = Self::start_db_tx(client)?;
        for transaction in transactions.iter() {
            let token_transfer = Self::new(transaction, token, indexer);
            token_transfer.add_to_db_tx(&mut db_tx)?;
        }
        match db_tx.commit() {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                Err(ModelError { reason: format!("During commit transaction in database occurred {e}") })
            }
        }
    }

    fn start_db_tx(client: &mut Client) -> Result<postgres::Transaction, ModelError>{
        match client.transaction() {
            Ok(db_tx) => {
               Ok(db_tx)
            },
            Err(e) => Err(ModelError {
                reason: format!("During preparing database transaction occurred {e}"),
            }),
        }
    }

    pub fn add_to_db_tx(&self, db_tx: &mut postgres::Transaction) -> Result<(), ModelError> {
        match db_tx.execute(
            INSERT_QUERY,
            &[
                &self.sender,
                &self.sender,
                &self.recipient,
                &self.tx_hash,
                &self.token_id,
                &self.amount,
                &self.token_instance_id,
                &self.fetched_by_id,
            ],
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

    pub fn new(transaction: &Transaction, token: &Token, indexer: &Indexer) -> Self {
        let mut result_amount = Decimal::ZERO;
        let mut result_token_id = Decimal::ZERO;
        match &transaction.transferred_token {
            Fungible { address, amount } => {
                result_amount = *amount;
            }
            NFT { address, token_id } => {
                result_token_id = *token_id;
            }
            ERC1155 {
                address,
                token_id,
                amount,
            } => {
                result_amount = *amount;
                result_token_id = *token_id;
            }
        }
        Self {
            id: 0,
            operator: transaction.sender.clone(),
            sender: transaction.sender.clone(),
            recipient: transaction.recipient.clone(),
            tx_hash: transaction.tx_hash.clone(),
            token_id: result_token_id,
            amount: result_amount,
            token_instance_id: token.id,
            fetched_by_id: indexer.id,
        }
    }
}
