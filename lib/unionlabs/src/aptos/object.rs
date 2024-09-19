use sha3::Digest;

use crate::hash::H256;

/// <https://legacy.aptos.dev/reference/move/?branch=mainnet&page=aptos-framework/doc/object.md#0x1_object_OBJECT_FROM_SEED_ADDRESS_SCHEME>
pub const OBJECT_FROM_SEED_ADDRESS_SCHEME: u8 = 254;

/// <https://legacy.aptos.dev/reference/move/?branch=mainnet&page=aptos-framework/doc/object.md#0x1_object_create_object_address>
#[must_use]
pub fn create_object_address(address: H256, seed: &[u8]) -> H256 {
    sha3::Sha3_256::new()
        .chain_update(address)
        .chain_update(seed)
        .chain_update([OBJECT_FROM_SEED_ADDRESS_SCHEME])
        .finalize()
        .into()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{aptos::object::create_object_address, hash::H256};

    #[test]
    fn ibc_store_address() {
        let address = H256::from(hex!(
            "dc08d8da6e3de03f62fdab618dc36f2e4e9cd70b03722c1d46df767370602771"
        ));

        let hash = create_object_address(address, b"Vault Seed Example");

        assert_eq!(
            <H256>::from(hex!(
                "14117a3adabf98fc2affa2f1583c201d348a6cb5791d94286987fffa49bb8bcb"
            )),
            hash
        );
    }
}
