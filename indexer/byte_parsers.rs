use rust_decimal::Decimal;
use web3::types::{H256, H160};

use super::commons::CycleError;

pub fn hex_string_to_bytes32(hex_string: &String) -> Result<H256, CycleError> {
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
        Err(e) => Err(CycleError {
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

pub fn bytes32_to_decimal(value: &H256) -> Result<Decimal, CycleError> {
    let mut hex_value = hex::encode(value.as_bytes());
    // remove trailing zeroes
    while hex_value.starts_with('0') {
        assert_eq!(hex_value.remove(0), '0');
    }
    // if every digit was '0' return just zero
    if hex_value.is_empty() {
        return Ok(Decimal::ZERO);
    }
    match Decimal::from_str_radix(hex_value.as_str(), 16) {
        Ok(result) => Ok(result),
        Err(e) => Err(CycleError {
            reason: format!("During converting bytes32 to Decimal {e} occurred"),
        }),
    }
}

pub fn bytes32_to_string(value: &H256) -> String {
    format!("0x{}", hex::encode(value.as_bytes()))
}
