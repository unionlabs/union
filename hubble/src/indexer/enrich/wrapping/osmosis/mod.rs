use alloy_primitives::{keccak256, U256};
use alloy_sol_types::SolValue;
use base58::ToBase58;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsmosisError {
    #[error("Failed to convert intermediate channel IDs to U256: invalid byte slice length")]
    InvalidChannelIdsLength,
    #[error("Invalid salt: salt is empty")]
    EmptySalt,
    #[error("Invalid salt: salt length {0} exceeds maximum length 64")]
    SaltTooLong(usize),
}

/// copy from: https://github.com/unionlabs/union/blob/f3abe0e1addd88325a39368699aaf2609c5fdfee/cosmwasm/osmosis-tokenfactory-token-minter/src/contract.rs#L227-L243
pub fn predict_osmosis_wrapper_0_1(
    intermediate_channel_ids: &[u8],
    receiver_channel_id: i64,
    original_token: &[u8],
    minter: &str,
) -> Result<Vec<u8>, OsmosisError> {
    let intermediate_channel_ids: U256 = U256::try_from_be_slice(intermediate_channel_ids)
        .ok_or(OsmosisError::InvalidChannelIdsLength)?;
    let params = (
        intermediate_channel_ids,
        receiver_channel_id,
        original_token,
    );
    let encoded = params.abi_encode_params();
    let salt: &[u8; 32] = &keccak256(encoded);

    if salt.is_empty() {
        return Err(OsmosisError::EmptySalt);
    }
    if salt.len() > 64 {
        return Err(OsmosisError::SaltTooLong(salt.len()));
    }

    let salt_base58 = salt.to_base58();

    Ok(format!("factory/{minter}/{salt_base58}")
        .as_bytes()
        .to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_address() -> Result<(), Box<dyn std::error::Error>> {
        // data for this test case obtain from  https://app.union.build/explorer/transfers/0x975aefd29f590a4416d211f5b269099158d3b155b2891634d1067f32c538a8bf
        let original_token = hex::decode("6d756e6f")?;

        // bech32-decoded deployer contract: "union1yl6hyqnuczg6828zkc7ntnge6cdnyf7dqmlwjkcn5xqp4pa09seqvut4nv"
        let deployer = "osmo13ulc6pqhm60qnx58ss7s3cft8cqfycexq3uy3dd2v0l8qsnkvk4sj22sn6";

        let receiver_channel_id: i64 = 1;

        let wrapped_token =
            predict_osmosis_wrapper_0_1(&[], receiver_channel_id, &original_token, deployer)?;

        // factory/osmo13ulc6pqhm60qnx58ss7s3cft8cqfycexq3uy3dd2v0l8qsnkvk4sj22sn6/G2NQNZUejTVx7ETxDoWh9nyd5X5ktnngmfT5eSVzBf5z
        let expected =
            hex::decode("666163746f72792f6f736d6f3133756c63367071686d3630716e78353873733773336366743863716679636578713375793364643276306c3871736e6b766b34736a3232736e362f47324e514e5a55656a54567837455478446f5768396e79643558356b746e6e676d6654356553567a4266357a")?;
        assert_eq!(wrapped_token, expected);
        Ok(())
    }

    #[test]
    fn test_invalid_channel_ids_length() {
        // Create a byte slice that's too long for U256 (more than 32 bytes)
        let invalid_path = vec![0u8; 33];
        let original_token = b"test";
        let minter = "test_minter";

        let result = predict_osmosis_wrapper_0_1(&invalid_path, 1, original_token, minter);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            OsmosisError::InvalidChannelIdsLength => {},
            _ => panic!("Expected InvalidChannelIdsLength error"),
        }
    }
}
