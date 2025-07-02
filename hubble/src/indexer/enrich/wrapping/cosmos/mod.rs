#![allow(dead_code)] // migrated from postgres; will refactor later
use alloy_primitives::U256;
use base58::ToBase58;
use sha3::{Digest, Keccak256};

pub fn predict_cosmos_wrapper_0_1(
    intermediate_channel_ids: &[u8],
    receiver_channel_id: i64,
    original_token: &[u8],
) -> Vec<u8> {
    let intermediate_channel_ids: U256 = U256::try_from_be_slice(intermediate_channel_ids)
        .expect("cannot convert intermediate_channel_ids to U256"); // handled by pgrx.

    let receiver_channel_id: u32 = receiver_channel_id
        .try_into()
        .expect("reciever channel can convert to u32");

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

    assert!(token_hash.len() <= MAX_DENOM_LENGTH);

    token_hash_base58.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_address() {
        let unwrapped_token = "muno".as_bytes();

        let unwrapped_token = predict_cosmos_wrapper_0_1(&[], 10, unwrapped_token);

        let expected = "FR6VWZZ45ePWQKhyxdtYfg1KFvmFcxsjikCtoKwYEoSj".as_bytes();
        assert_eq!(unwrapped_token, expected);
    }

    #[test]
    fn test_cosmos_to_cosmos() {
        // transfer from babylon to union: send tx: 0x9dea64237ef01a1d0ccc58b903e13a58f9ed223fcf40064db775e8d8981f404e
        let unwrapped_token = hex::decode("62626e316c6571716d71306c706c36637a6664643574766b7261686779396a6e663964756b71797838336c6b773530727037343561396d71386e78307864").unwrap();

        let unwrapped_token = predict_cosmos_wrapper_0_1(&[], 22, &unwrapped_token);

        let expected = hex::decode("39727163777967743377745777466378564e374d66625865514e7a58475943486d77484b4e576938684b5137").unwrap();
        assert_eq!(unwrapped_token, expected);
    }
}
