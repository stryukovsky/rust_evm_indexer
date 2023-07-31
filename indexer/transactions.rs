use std::fmt::Display;

use rust_decimal::Decimal;

pub enum TransferredToken {
    Fungible{address: String, amount: Decimal},
    NFT{address: String, token_id: Decimal},
    ERC1155{address: String, token_id: Decimal, amount: Decimal},
}

pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub tx_hash: String,
    pub transferred_token: TransferredToken,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} -> {} \n", self.tx_hash, self.sender, self.recipient)?;
        match &self.transferred_token {
            TransferredToken::Fungible{address, amount} => {
                write!(f, "                                                          fungible {address} amount {amount}")
            }
            TransferredToken::NFT { address, token_id } => {
                write!(f, "NFT {address} with id {token_id}")
            },
            TransferredToken::ERC1155 { address, token_id, amount } => {
                write!(f, "ERC1155 token {address} with id {token_id} amount {amount}")
            }
        }
    }
}
