use std::fmt::Display;

use rust_decimal::Decimal;

pub enum TransferredToken {
    Fungible{address: String, amount: Decimal},
    NFT{address: String, token_id: Decimal},
    ERC1155{address: String, token_id: Decimal, amount: Decimal},
}

pub struct Transfer {
    pub sender: String,
    pub recipient: String,
    pub tx_hash: String,
    pub transferred_token: TransferredToken,
}

impl Display for Transfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At {} transfer: {} -> {} of ", self.tx_hash, self.sender, self.recipient)?;
        match &self.transferred_token {
            TransferredToken::Fungible{address, amount} => {
                write!(f, "fungible token {} amount {}", address, amount)
            }
            TransferredToken::NFT { address, token_id } => todo!(),
            TransferredToken::ERC1155 { address, token_id, amount } => todo!(),
        }
    }
}
