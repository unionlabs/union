#![allow(dead_code)] // migrated from postgres; will refactor later
use alloy_primitives::U256;
use sha3::{Digest, Sha3_256};

// https://github.com/unionlabs/union/blob/main/aptos/ucs03-zkgm/sources/zkgm.move#L273
pub fn predict_aptos_wrapper_0_1(
    intermediate_channel_ids: &[u8],
    receiver_channel_id: i64,
    original_token: &[u8],
    creator: &[u8],
) -> Vec<u8> {
    let intermediate_channel_ids: U256 = U256::try_from_be_slice(intermediate_channel_ids)
        .expect("cannot convert intermediate_channel_ids to U256"); // handled by pgrx.

    let receiver_channel_id: u32 = receiver_channel_id
        .try_into()
        .expect("reciever channel can convert to u32");

    let salt: [u8; 32] = Sha3_256::new()
        .chain_update(
            [
                intermediate_channel_ids.to_le_bytes_vec().as_ref(),
                receiver_channel_id.to_le_bytes().as_ref(),
                original_token,
            ]
            .concat(),
        )
        .finalize()
        .into();

    let vault_address = get_vault_address(creator);

    create_object_address(&vault_address, &salt).to_vec()
}

// https://github.com/unionlabs/union/blob/main/aptos/ucs03-zkgm/sources/zkgm.move#L191-L193
fn get_vault_address(creator: &[u8]) -> [u8; 32] {
    create_object_address(creator, b"ibc-union-app-v1")
}

// DeriveObjectAddressFromSeed https://github.com/aptos-labs/aptos-core/blob/8787bb0e9ca2f18969e61ab8e5578201c0e4956a/types/src/transaction/authenticator.rs#L495
const DERIVE_OBJECT_ADDRESS_FROM_SEED: u8 = 254;

// https://github.com/aptos-labs/aptos-core/blob/72f04e09def0f6ed796a00a83b1b52fa31a5e860/types/src/account_address.rs#L176
fn create_object_address(creator: &[u8], seed: &[u8]) -> [u8; 32] {
    let mut input = creator.to_vec();
    input.extend(seed);
    input.push(DERIVE_OBJECT_ADDRESS_FROM_SEED);

    let mut sha3 = Sha3_256::new();
    sha3.update(input);
    sha3.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // data from https://app.union.build/explorer/transfers/0xCE5CF961148C117E92E7BB824C4EFCDC783BFE873799556CBCD53947C79A4CF4
    #[test]
    fn test_on_chain_example() {
        let path = [];
        let destination_channel_id = 2;
        let unwrapped_token = hex::decode("6d756e6f").unwrap();
        let creator =
            hex::decode("80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84")
                .unwrap();
        let expected =
            hex::decode("188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c")
                .unwrap();

        let actual =
            predict_aptos_wrapper_0_1(&path, destination_channel_id, &unwrapped_token, &creator);

        assert_eq!(actual, expected);
    }
}
