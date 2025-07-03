use alloy_primitives::{keccak256, U256};
use alloy_sol_types::SolValue;
use hex_literal::hex;
use sha2::{digest::Update, Digest};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Instantiate2Error {
    #[error("Failed to convert intermediate channel IDs to U256: invalid byte slice length")]
    InvalidChannelIdsLength,
    #[error("Invalid salt: salt is empty")]
    EmptySalt,
    #[error("Invalid salt: salt length {0} exceeds maximum length 64")]
    SaltTooLong(usize),
}

/// copy from: https://docs.rs/cosmwasm-std/2.1.4/src/cosmwasm_std/addresses.rs.html#308-317
pub fn instantiate2_0_1(
    intermediate_channel_ids: &[u8],
    receiver_channel_id: i64,
    original_token: &[u8],
    creator: &[u8],
) -> Result<Vec<u8>, Instantiate2Error> {
    // based on the dummy contract
    const CHECKSUM: &[u8; 32] =
        &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

    const MSG: &[u8] = b"";

    let intermediate_channel_ids: U256 = U256::try_from_be_slice(intermediate_channel_ids)
        .ok_or(Instantiate2Error::InvalidChannelIdsLength)?;
    let params = (
        intermediate_channel_ids,
        receiver_channel_id,
        original_token,
    );
    let encoded = params.abi_encode_params();
    let salt: &[u8; 32] = &keccak256(encoded);

    if salt.is_empty() {
        return Err(Instantiate2Error::EmptySalt);
    }
    if salt.len() > 64 {
        return Err(Instantiate2Error::SaltTooLong(salt.len()));
    }

    let mut key = Vec::<u8>::new();
    key.extend_from_slice(b"wasm\0");
    key.extend_from_slice(&(CHECKSUM.len() as u64).to_be_bytes());
    key.extend_from_slice(CHECKSUM);
    key.extend_from_slice(&(creator.len() as u64).to_be_bytes());
    key.extend_from_slice(creator);
    key.extend_from_slice(&(salt.len() as u64).to_be_bytes());
    key.extend_from_slice(salt);
    key.extend_from_slice(&(MSG.len() as u64).to_be_bytes());
    key.extend_from_slice(MSG);
    Ok(hash("module", &key))
}

/// The "Basic Address" Hash from
/// https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/docs/architecture/adr-028-public-key-addresses.md
fn hash(ty: &str, key: &[u8]) -> Vec<u8> {
    let inner = sha2::Sha256::digest(ty.as_bytes());
    sha2::Sha256::new()
        .chain(inner)
        .chain(key)
        .finalize()
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_address() -> Result<(), Box<dyn std::error::Error>> {
        // data for this test case obtain from  https://app.union.build/explorer/transfers/0x975aefd29f590a4416d211f5b269099158d3b155b2891634d1067f32c538a8bf
        let original_token = hex::decode("685ce6742351ae9b618f383883d6d1e0c5a31b4b")?;

        // bech32-decoded deployer contract: "union1yl6hyqnuczg6828zkc7ntnge6cdnyf7dqmlwjkcn5xqp4pa09seqvut4nv"
        let deployer =
            hex::decode("27F572027CC091A3A8E2B63D35CD19D61B3227CD06FEE95B13A1801A87AF2C32")?;

        let receiver_channel_id: i64 = 1;

        let wrapped_token = instantiate2_0_1(&[], receiver_channel_id, &original_token, &deployer)?;

        // bech32-decoded quote-token: union1surgyrm5xwfwughm6rfv76kd6vm2fc8vgpxxd6k6su6xsrxz0jgs7w967n
        // 0x8706820F743392EE22FBD0D2CF6ACDD336A4E0EC404C66EADA8734680CC27C91
        let expected =
            hex::decode("8706820F743392EE22FBD0D2CF6ACDD336A4E0EC404C66EADA8734680CC27C91")?;
        assert_eq!(wrapped_token, expected);
        Ok(())
    }

    #[test]
    fn test_u256_conversion() -> Result<(), Box<dyn std::error::Error>> {
        assert_decode_encode_equals("0x0")?;
        assert_decode_encode_equals("0x10203")?;
        assert_decode_encode_equals("0x1234567890abcdef")?;
        assert_decode_encode_equals("0xfedcba0987654321")?;
        assert_decode_encode_equals("0x71afd498d0000")?;
        Ok(())
    }

    fn assert_decode_encode_equals(hex_0x: &str) -> Result<(), Box<dyn std::error::Error>> {
        let hex = &hex_0x[2..];
        let hex_even_nibbles = match hex.len() % 2 == 0 {
            true => hex.to_string(),
            false => format!("0{}", hex),
        };
        let u8_vec = hex::decode(hex_even_nibbles)?;
        let u8_array = u8_vec.as_slice();

        let u256 = U256::try_from_be_slice(u8_array).ok_or("Failed to convert to U256")?;
        let json_value = serde_json::to_value(u256)?;
        let json_string = json_value.as_str().ok_or("Failed to convert to string")?;

        assert_eq!(hex_0x, json_string);
        Ok(())
    }

    #[test]
    fn test_invalid_channel_ids_length() {
        // Create a byte slice that's too long for U256 (more than 32 bytes)
        let invalid_path = vec![0u8; 33];
        let original_token = b"test";
        let creator = b"test_creator";

        let result = instantiate2_0_1(&invalid_path, 1, original_token, creator);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            Instantiate2Error::InvalidChannelIdsLength => {},
            _ => panic!("Expected InvalidChannelIdsLength error"),
        }
    }
}
