use std::fmt::Display;

pub enum IndexerStrategy {
    Recipient,
    Sender,
    TokenScan,
}
const TOKEN_STRATEGY_RECIPIENT: &'static str = "recipient";
const TOKEN_STRATEGY_SENDER: &'static str = "sender";
const TOKEN_STRATEGY_TOKEN_SCAN: &'static str = "token_scan";

impl Display for IndexerStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipient => write!(f, "{}", TOKEN_STRATEGY_RECIPIENT),
            Self::Sender => write!(f, "{}", TOKEN_STRATEGY_SENDER),
            Self::TokenScan => write!(f, "{}", TOKEN_STRATEGY_TOKEN_SCAN),
            _ => Err(std::fmt::Error),
        }
    }
}

impl From<&String> for IndexerStrategy {
    fn from(value: &String) -> Self {
        match value.as_str() {
            TOKEN_STRATEGY_RECIPIENT => Self::Recipient,
            TOKEN_STRATEGY_SENDER => Self::Sender,
            TOKEN_STRATEGY_TOKEN_SCAN => Self::TokenScan,
            _ => panic!("Not implemented IndexerStrategy"),
        }
    }
}
