#![allow(dead_code)] // migrated from postgres; will refactor later
use alloy_primitives::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Erc55Error {
    #[error("Invalid address length: expected 20 bytes, got {0}")]
    InvalidAddressLength(usize),
}

/// specification: https://eips.ethereum.org/EIPS/eip-55
pub fn erc55_to_checksum_0_1(address: &[u8]) -> Result<String, Erc55Error> {
    let address: Address = address
        .try_into()
        .map_err(|_| Erc55Error::InvalidAddressLength(address.len()))?;

    Ok(address.to_checksum(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testcases_from_spec() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            // All caps
            "0x52908400098527886E0F7030069857D2E4169EE7",
            "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
            // All Lower
            "0xde709f2102306220921060314715629080e2fb77",
            "0x27b1fdb04752bbc536007a920d24acb045561c26",
            // Normal
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ];

        for &address in &test_cases {
            test(address)?;
        }
        Ok(())
    }

    fn test(expected: &str) -> Result<(), Box<dyn std::error::Error>> {
        let address = hex::decode(expected.trim_start_matches("0x"))?;
        let checksum = erc55_to_checksum_0_1(&address)?;
        assert_eq!(checksum, expected);
        Ok(())
    }

    #[test]
    fn test_invalid_address_length() {
        // Test with address that's too short (19 bytes instead of 20)
        let invalid_address = vec![0u8; 19];
        let result = erc55_to_checksum_0_1(&invalid_address);

        assert!(result.is_err());
        match result.unwrap_err() {
            Erc55Error::InvalidAddressLength(len) => assert_eq!(len, 19),
        }

        // Test with address that's too long (21 bytes instead of 20)
        let invalid_address = vec![0u8; 21];
        let result = erc55_to_checksum_0_1(&invalid_address);

        assert!(result.is_err());
        match result.unwrap_err() {
            Erc55Error::InvalidAddressLength(len) => assert_eq!(len, 21),
        }
    }
}
