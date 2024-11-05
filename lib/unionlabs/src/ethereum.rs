use sha2::Digest;
use sha3::Keccak256;

use crate::{
    ethereum::slot::{MappingKey, Slot},
    hash::H256,
    uint::U256,
};

pub mod slot;

#[inline]
#[must_use]
pub fn keccak256(bytes: impl AsRef<[u8]>) -> H256 {
    Keccak256::new().chain_update(bytes).finalize().into()
}

/// The slot of the `mapping(bytes32 => bytes32) public commitments` mapping in the `IBCStore` contract.
pub const IBC_HANDLER_COMMITMENTS_SLOT: U256 = U256::from_limbs([0, 0, 0, 0]);

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: `keccak256(keccak256(abi.encode_packed(path)) || slot)`
#[must_use = "calculating the commitment key has no effect"]
pub fn ibc_commitment_key(path: H256) -> U256 {
    Slot::Mapping(
        &Slot::Offset(IBC_HANDLER_COMMITMENTS_SLOT),
        MappingKey::Bytes32(path),
    )
    .slot()
}

// #[cfg(test)]
// mod tests {
//     use hex_literal::hex;

//     use super::*;
//     use crate::{ics24::ethabi, id::ConnectionId};

//     #[test]
//     fn commitment_key() {
//         let commitments = [
//             (
//                 U256::from_be_bytes(hex!(
//                     "55c4893838cf8a468bfdb0c63e25a4c924d9b7ad283fc335d5f527d29b2fcfc7"
//                 )),
//                 ConnectionId::new(100),
//                 0,
//             ),
//             (
//                 U256::from_be_bytes(hex!(
//                     "f39538e1f0ca1c5f5ecdf1bb05f67c173f2d0f75b41fbb5be884f6aab2ebae91"
//                 )),
//                 ConnectionId::new(1),
//                 5,
//             ),
//         ];

//         for (expected, connection_id, slot) in commitments {
//             assert_eq!(
//                 ibc_commitment_key(ethabi::connection_key(connection_id.id()), U256::from(slot)),
//                 expected
//             );
//         }
//     }
// }
