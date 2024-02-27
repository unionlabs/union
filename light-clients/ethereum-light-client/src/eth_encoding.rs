use sha3::Digest;
use unionlabs::{hash::H256, uint::U256};

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: keccak256(keccak256(abi.encode_packed(path)) || slot)
pub fn generate_commitment_key(path: &str, slot: U256) -> H256 {
    sha3::Keccak256::new()
        .chain_update(sha3::Keccak256::new().chain_update(path).finalize())
        .chain_update(slot.to_big_endian())
        .finalize()
        .into()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::proof::ConnectionPath;

    use super::*;

    #[test]
    fn gen_commitment_key() {
        let commitments = [
            (
                hex!("55c4893838cf8a468bfdb0c63e25a4c924d9b7ad283fc335d5f527d29b2fcfc7"),
                "connection-100",
                0,
            ),
            (
                hex!("1c88d50be829300c4b7ae8cca3894ad830043b9e9004e81af6b7d1d36eb80611"),
                "hellowolddd",
                5,
            ),
        ];

        for (expected, connection_id, slot) in commitments {
            assert_eq!(
                generate_commitment_key(
                    &ConnectionPath {
                        connection_id: unionlabs::validated::Validated::new(
                            connection_id.to_string()
                        )
                        .unwrap(),
                    }
                    .to_string(),
                    U256::from(slot),
                )
                .as_ref(),
                &expected
            );
        }
    }
}
