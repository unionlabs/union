use sha3::Digest;
use unionlabs::{hash::H256, uint::U256};

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: keccak256(keccak256(abi.encode_packed(path)) || slot)
pub fn generate_commitment_key(path: &str, slot: U256) -> H256 {
    sha3::Keccak256::new()
        .chain_update(sha3::Keccak256::new().chain_update(path).finalize())
        .chain_update(slot.to_be_bytes())
        .finalize()
        .into()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::ics24::ConnectionPath;

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
                hex!("f39538e1f0ca1c5f5ecdf1bb05f67c173f2d0f75b41fbb5be884f6aab2ebae91"),
                "connection-1",
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
