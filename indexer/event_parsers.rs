use super::{
    byte_parsers::{bytes20_to_address, bytes32_to_address, bytes32_to_uint256, bytes32_to_usize, bytes32_to_string, bytes_to_uint256_array},
    commons::IndexerError,
    transactions::{Transaction, TransferredToken},
};
use crate::models::{Token, TokenType, ERC1155_TRANSFER_BATCH, ERC1155_TRANSFER_SINGLE};
use web3::types::{Log, H256};


pub trait EventParser {
    fn parse(&self, event: &Log) -> Result<Transaction, IndexerError>;
}

fn get_event_address(event: &Log, target_token: &Token) -> Result<String, IndexerError> {
    let address = bytes20_to_address(&event.address);
    if address.to_lowercase() == target_token.address.to_lowercase() {
        Ok(address)
    } else {
        Err(IndexerError {
            reason: format!(
                "Mismatch: parser target token is {} but in event address is {address}",
                target_token.address
            ),
        })
    }
}

fn get_event_tx_hash(event: &Log) -> Result<String, IndexerError> {
    if event.transaction_hash.is_none() {
        Err(IndexerError {
            reason: String::from("Event has no tx hash. abort"),
        })
    } else {
        let transaction_hash = bytes32_to_string(&event.transaction_hash.unwrap());
        Ok(transaction_hash)
    }
}

fn get_event_topics_length(event: &Log) -> Result<usize, IndexerError> {
    let length = event.topics.len();
    if length < 3 {
        return Err(IndexerError {
            reason: format!("Bad event. Expected at least 3 topics, actual {length}"),
        });
    }
    if length > 4 {
        return Err(IndexerError {
            reason: format!("Bad event. Expected 3 or 4 topics, actual {length}"),
        });
    }
    return Ok(length);
}

fn get_event_signature(event: &Log, target_token: &Token) -> Result<H256, IndexerError> {
    let topic = event.topics.get(0).unwrap();
    let target_signatures = target_token.token_type.get_events_hashes();
    if target_signatures.contains(topic) {
        Ok(*topic)
    } else {
        Err(IndexerError { reason: format!("Token {} should accept event(s) with signature(s) {target_signatures:?}, encountered event with signature {topic}", target_token.name) })
    }
}

fn get_event_participants(event: &Log) -> Result<(String, String), IndexerError> {
    let sender;
    match event.topics.get(1) {
        Some(raw_sender) => {
            sender = bytes32_to_address(raw_sender);
        }
        None => {
            return Err(IndexerError {
                reason: format!("Sender not found"),
            });
        }
    }
    let recipient;
    match event.topics.get(2) {
        Some(raw_recipient) => {
            recipient = bytes32_to_address(raw_recipient);
        }
        None => {
            return Err(IndexerError {
                reason: format!("Recipient not found"),
            });
        }
    }
    Ok((sender, recipient))
}

pub struct FungibleEventParser<'a> {
    target_token: &'a Token,
}

impl<'a> EventParser for FungibleEventParser<'a> {
    fn parse(&self, event: &Log) -> Result<Transaction, IndexerError> {
        let address = get_event_address(event, self.target_token)?;
        let tx_hash = get_event_tx_hash(event)?;
        let topics_count = get_event_topics_length(event)?;
        let event_signature = get_event_signature(event, self.target_token)?;
        let (sender, recipient) = get_event_participants(event)?;
        let source_for_amount: H256; // if amount is indexed then it is in topics; otherwise in event data
        if topics_count == 3 && event.data.0.len() == 32 {
            let data = H256::from_slice(event.data.0.as_slice());
            source_for_amount = data;
        } else if topics_count == 4 {
            source_for_amount = *event.topics.get(3).unwrap();
        } else {
            return Err(IndexerError { reason: format!("Bad event {tx_hash}: expected either 3 topics with data or 4 topics with no data") });
        }
        let amount = bytes32_to_uint256(&source_for_amount)?;
        Ok(Transaction {
            sender,
            recipient,
            tx_hash,
            transferred_token: TransferredToken::Fungible { amount, address },
        })
    }
}

pub struct NFTEventParser<'a> {
    pub target_token: &'a Token,
}

impl<'a> EventParser for NFTEventParser<'a> {
    fn parse(&self, event: &Log) -> Result<Transaction, IndexerError> {
        let address = get_event_address(event, self.target_token)?;
        let tx_hash = get_event_tx_hash(event)?;
        let topics_count = get_event_topics_length(event)?;
        let event_signature = get_event_signature(event, self.target_token)?;
        let (sender, recipient) = get_event_participants(event)?;
        let source_for_token_id: H256; // if amount is indexed then it is in topics; otherwise in event data
        if topics_count == 3 && event.data.0.len() == 32 {
            let data = H256::from_slice(event.data.0.as_slice());
            source_for_token_id = data;
        } else if topics_count == 4 {
            source_for_token_id = *event.topics.get(3).unwrap();
        } else {
            return Err(IndexerError { reason: format!("Bad event {tx_hash}: expected either 3 topics with data or 4 topics with no data") });
        }
        let token_id = bytes32_to_uint256(&source_for_token_id)?;
        Ok(Transaction {
            sender,
            recipient,
            tx_hash,
            transferred_token: TransferredToken::NFT { address, token_id },
        })
    }
}

pub struct ERC1155EventParser<'a> {
    pub target_token: &'a Token,
}

impl<'a> EventParser for ERC1155EventParser<'a> {
    fn parse(&self, event: &Log) -> Result<Transaction, IndexerError> {
        let address = get_event_address(event, self.target_token)?;
        let tx_hash = get_event_tx_hash(event)?;
        let topics_count = get_event_topics_length(event)?;
        let (sender, recipient) = get_event_participants(event)?;
        let event_signature = get_event_signature(event, self.target_token)?;
        let data = event.data.0.as_slice();
        if event_signature.as_bytes() == web3::signing::keccak256(ERC1155_TRANSFER_SINGLE) {
            if data.len() != 64 {
                return Err(IndexerError {
                    reason: format!(
                        "ERC1155 TransferSingle at {tx_hash} expected 64 bytes for data, found {}",
                        data.len()
                    ),
                });
            }
            let token_id_raw = H256::from_slice(&data[0..32]);
            let amount_raw = H256::from_slice(&data[32..]);
            let token_id = bytes32_to_uint256(&token_id_raw)?;
            let amount = bytes32_to_uint256(&amount_raw)?;
            Ok(Transaction {
                sender,
                recipient,
                tx_hash,
                transferred_token: TransferredToken::ERC1155 {
                    address,
                    token_ids: vec![token_id],
                    amounts: vec![amount],
                },
            })
        } else if event_signature.as_bytes() == web3::signing::keccak256(ERC1155_TRANSFER_BATCH) {
            if data.len() < 64 || data.len() % 32 != 0 {
                return Err(IndexerError {
                    reason: format!(
                        "ERC1155 TransferSingle at {tx_hash} expected at least 64 bytes for data, found {}",
                        data.len()
                    ),
                });
            }
            let token_ids_location_raw = H256::from_slice(&data[0..32]);
            let amounts_location_raw = H256::from_slice(&data[32..64]);
            let token_ids_location = bytes32_to_usize(&token_ids_location_raw)?;
            let amounts_location =  bytes32_to_usize(&amounts_location_raw)?;
            let token_ids = bytes_to_uint256_array(data, token_ids_location)?;
            let amounts = bytes_to_uint256_array(data, amounts_location)?;
            Ok(Transaction { sender, recipient, tx_hash, transferred_token: TransferredToken::ERC1155 { address, token_ids, amounts } })
        } else {
            Err(IndexerError { reason: String::from("Bad event signature") })
        }
    }
}

pub fn get_event_parser<'a>(token: &'a Token) -> Box<dyn EventParser + 'a> {
    match &token.token_type {
        TokenType::ERC20 => Box::new(FungibleEventParser {
            target_token: token,
        }),
        TokenType::ERC721 => Box::new(NFTEventParser {
            target_token: token,
        }),
        TokenType::ERC1155 => todo!(),
    }
}
