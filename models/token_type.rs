use std::fmt::Display;

use web3::types::H256;


pub enum TokenType {
    ERC20,
    ERC721,
    ERC1155,
}
const TOKEN_TYPE_ERC20: &'static str = "erc20";
const TOKEN_TYPE_ERC721: &'static str = "erc721";
const TOKEN_TYPE_ERC1155: &'static str = "erc1155";

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ERC20 => write!(f, "{}", TOKEN_TYPE_ERC20),
            Self::ERC721 => write!(f, "{}", TOKEN_TYPE_ERC721),
            Self::ERC1155 => write!(f, "{}", TOKEN_TYPE_ERC1155),
        }
    }
}

impl From<&String> for TokenType {
    fn from(value: &String) -> Self {
        match value.as_str() {
            TOKEN_TYPE_ERC20 => Self::ERC20,
            TOKEN_TYPE_ERC721 => Self::ERC721,
            TOKEN_TYPE_ERC1155 => Self::ERC1155,
            _ => panic!("Not implemented TokenType"),
        }
    }
}

impl TokenType {
    pub fn get_events_hashes(&self) -> Vec<H256> {
        match self {
            Self::ERC20 => vec![H256::from_slice(&web3::signing::keccak256(
                b"Transfer(address,address,uint256)",
            ))],
            Self::ERC721 => vec![H256::from_slice(&web3::signing::keccak256(
                b"Transfer(address,address,uint256)",
            ))],
            Self::ERC1155 => vec![
                H256::from_slice(&web3::signing::keccak256(
                    b"TransferSingle(address,address,uint256,uint256)",
                )),
                H256::from_slice(&web3::signing::keccak256(
                    b"TransferBatch(address,address,uint256[],uint256[])",
                )),
            ],
        }
    }
}
