use rust_decimal::Decimal;
use web3::types::{H256, H160, U256};

use super::commons::IndexerError;

pub fn hex_string_to_bytes32(hex_string: &String) -> Result<H256, IndexerError> {
    let mut source = hex_string.clone();
    if source.starts_with("0x") {
        assert_eq!(source.remove(0), '0');
        assert_eq!(source.remove(0), 'x');
    }
    match hex::decode(source) {
        Ok(mut bytes) => {
            let desired_length = 32;
            while bytes.len() < desired_length {
                bytes.insert(0, 0);
            }
            let result = H256::from_slice(bytes.as_slice());
            Ok(result)
        }
        Err(e) => Err(IndexerError {
            reason: e.to_string(),
        }),
    }
}

pub fn bytes32_to_address(value: &H256) -> String {
    let address_bytes = &value[12..];
    format!("0x{}", hex::encode(address_bytes))
}

pub fn bytes20_to_address(value: &H160) -> String {
    format!("0x{}", hex::encode(value))
}

pub fn bytes32_to_uint256(value: &H256) -> Result<U256, IndexerError> {
    let mut hex_value = hex::encode(value.as_bytes());
    // remove trailing zeroes
    while hex_value.starts_with('0') {
        assert_eq!(hex_value.remove(0), '0');
    }
    // if every digit was '0' return just zero
    if hex_value.is_empty() {
        return Ok(U256::zero());
    }
    match U256::from_str_radix(hex_value.as_str(), 16) {
        Ok(result) => Ok(result),
        Err(e) => Err(IndexerError {
            reason: format!("During converting bytes32 to Decimal {e} occurred"),
        }),
    }
}

pub fn bytes32_to_usize(value: &H256) -> Result<usize, IndexerError> {
    match usize::from_str_radix(value.to_string().as_str(), 10) {
        Ok(result) => {
            Ok(result)
        },
        Err(e) => {
            Err(IndexerError { reason: format!("During converting from string to u128 occurred {e}") })
        }
    }
}

pub fn bytes32_to_string(value: &H256) -> String {
    format!("0x{}", hex::encode(value.as_bytes()))
}

pub fn bytes_to_uint256_array(value: &[u8], location: usize) -> Result<Vec<U256>, IndexerError> {
    let raw_length = &H256::from_slice(&value[location..location+32]);
    let length = bytes32_to_usize(&raw_length)?;
    let mut result = vec![];
    for i in 1..length {
        let raw_uint256_value = &value[i*32..(i+1)*32];
        result.push(bytes32_to_uint256(&H256::from_slice(raw_uint256_value))?);
    }
    Ok(result)
}
