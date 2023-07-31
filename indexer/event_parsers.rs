use super::{
    byte_parsers::{bytes20_to_address, bytes32_to_address, bytes32_to_decimal, bytes32_to_string},
    commons::CycleError,
    transactions::{Transaction, TransferredToken},
};
use crate::models::{Token, TokenType};
use web3::types::{Log, H256};

pub trait EventParser {
    fn parse(&self, event: &Log) -> Result<Transaction, CycleError>;
}

pub struct FungibleEventParser<'a> {
    target_token: &'a Token,
}

impl<'a> EventParser for FungibleEventParser<'a> {
    fn parse(&self, event: &Log) -> Result<Transaction, CycleError> {
        let address = bytes20_to_address(&event.address);
        if address != self.target_token.address {
            return Err(CycleError {
                reason: format!(
                    "Mismatch: parser target token is {} but in event address is {address}",
                    self.target_token.address
                ),
            });
        }
        if event.transaction_hash.is_none() {
            return Err(CycleError {
                reason: String::from("Event has no tx hash. abort"),
            });
        }
        let transaction_hash = bytes32_to_string(&event.transaction_hash.unwrap());
        let topics_count = event.topics.len();
        if topics_count < 3 {
            return Err(CycleError{
                reason: format!("Bad event in tx {transaction_hash:?}. Expected at least 3 topics, actual {topics_count}")
            });
        }
        let sender = event.topics.get(1).unwrap();
        let recipient = event.topics.get(2).unwrap();
        let source_for_amount: H256; // if amount is indexed then it is in topics; otherwise in event data
        if topics_count == 3 && event.data.0.len() == 32 {
            let data = H256::from_slice(event.data.0.as_slice());
            source_for_amount = data;
        } else if topics_count == 4 {
            source_for_amount = *event.topics.get(3).unwrap();
        } else {
            return Err(CycleError { reason: String::from("Bad topics length: expected either 3 topics with data or 4 topics with no data") });
        }
        let amount = bytes32_to_decimal(&source_for_amount)?;
        Ok(Transaction {
            sender: bytes32_to_address(sender),
            recipient: bytes32_to_address(recipient),
            tx_hash: transaction_hash,
            transferred_token: TransferredToken::Fungible { amount, address },
        })
    }
}

pub fn get_event_parser<'a>(token: &'a Token) -> Box<dyn EventParser + 'a> {
    match TokenType::from(&token.token_type) {
        TokenType::ERC20 => Box::new(FungibleEventParser {
            target_token: token,
        }),
        TokenType::ERC721 => todo!(),
        TokenType::ERC1155 => todo!(),
    }
}
