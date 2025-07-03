#![allow(dead_code)] // migrated from postgres; will refactor later
use alloy_primitives::U256;
use base58::ToBase58;
use sha3::{Digest, Keccak256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CosmosError {
    #[error("Failed to convert intermediate channel IDs to U256: invalid byte slice length")]
    InvalidChannelIdsLength,
    #[error("Failed to convert receiver channel ID {0} to u32: value out of range")]
    InvalidReceiverChannelId(i64),
    #[error("Token hash length {0} exceeds maximum denom length {1}")]
    TokenHashTooLong(usize, usize),
}

pub fn predict_cosmos_wrapper_0_1(
    intermediate_channel_ids: &[u8],
    receiver_channel_id: i64,
    original_token: &[u8],
) -> Result<Vec<u8>, CosmosError> {
    let intermediate_channel_ids: U256 = U256::try_from_be_slice(intermediate_channel_ids)
        .ok_or(CosmosError::InvalidChannelIdsLength)?;

    let receiver_channel_id: u32 = receiver_channel_id
        .try_into()
        .map_err(|_| CosmosError::InvalidReceiverChannelId(receiver_channel_id))?;

    const MAX_DENOM_LENGTH: usize = 44;

    let token_hash: [u8; 32] = Keccak256::new()
        .chain_update(
            [
                intermediate_channel_ids.to_be_bytes_vec().as_ref(),
                receiver_channel_id.to_be_bytes().as_ref(),
                original_token,
            ]
            .concat(),
        )
        .finalize()
        .into();

    let token_hash_base58 = token_hash.to_base58();

    if token_hash_base58.len() > MAX_DENOM_LENGTH {
        return Err(CosmosError::TokenHashTooLong(token_hash_base58.len(), MAX_DENOM_LENGTH));
    }

    Ok(token_hash_base58.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_address() -> Result<(), Box<dyn std::error::Error>> {
        let unwrapped_token = "muno".as_bytes();

        let unwrapped_token = predict_cosmos_wrapper_0_1(&[], 10, unwrapped_token)?;

        let expected = "FR6VWZZ45ePWQKhyxdtYfg1KFvmFcxsjikCtoKwYEoSj".as_bytes();
        assert_eq!(unwrapped_token, expected);
        Ok(())
    }

    #[test]
    fn test_cosmos_to_cosmos() -> Result<(), Box<dyn std::error::Error>> {
        // transfer from babylon to union: send tx: 0x9dea64237ef01a1d0ccc58b903e13a58f9ed223fcf40064db775e8d8981f404e
        let unwrapped_token = hex::decode("62626e316c6571716d71306c706c36637a6664643574766b7261686779396a6e663964756b71797838336c6b773530727037343561396d71386e78307864")?;

        let unwrapped_token = predict_cosmos_wrapper_0_1(&[], 22, &unwrapped_token)?;

        let expected = hex::decode("39727163777967743377745777466378564e374d66625865514e7a58475943486d77484b4e576938684b5137")?;
        assert_eq!(unwrapped_token, expected);
        Ok(())
    }

    #[test]
    fn test_invalid_channel_id() {
        let unwrapped_token = b"test";
        let invalid_channel_id = -1i64; // Negative value should fail conversion to u32

        let result = predict_cosmos_wrapper_0_1(&[], invalid_channel_id, unwrapped_token);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CosmosError::InvalidReceiverChannelId(id) => assert_eq!(id, -1),
            _ => panic!("Expected InvalidReceiverChannelId error"),
        }
    }

    #[test]
    fn test_invalid_channel_ids_length() {
        // Create a byte slice that's too long for U256 (more than 32 bytes)
        let invalid_path = vec![0u8; 33];
        let unwrapped_token = b"test";

        let result = predict_cosmos_wrapper_0_1(&invalid_path, 1, unwrapped_token);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CosmosError::InvalidChannelIdsLength => {},
            _ => panic!("Expected InvalidChannelIdsLength error"),
        }
    }

    #[test]
    fn test_token_hash_length_validation() {
        // Note: This test might be hard to trigger in practice since the hash is always 32 bytes
        // and base58 encoding typically produces strings shorter than 44 characters.
        // But the validation is there for safety, so we document the test case.
        
        // For now, let's test that normal cases don't trigger the error
        let unwrapped_token = b"test_token_that_should_work_fine";
        let result = predict_cosmos_wrapper_0_1(&[], 1, unwrapped_token);
        
        assert!(result.is_ok(), "Normal token should not trigger length error");
        
        let token_hash = result.unwrap();
        assert!(token_hash.len() <= 44, "Result should be within length limits");
    }
}
