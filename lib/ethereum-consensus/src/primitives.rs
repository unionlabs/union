pub use crate::crypto::{PublicKey as BlsPublicKey, Signature as BlsSignature};
pub use crate::domains::DomainType;
use crate::ssz::ByteVector;
use ssz_rs::prelude::*;

pub use ssz_rs::prelude::U256;

pub type Root = Node;
pub type Slot = u64;
pub type Epoch = u64;

pub type CommitteeIndex = usize;
pub type ValidatorIndex = usize;
pub type WithdrawalIndex = usize;
pub type Gwei = u64;
pub type Hash32 = Bytes32;

pub type Version = [u8; 4];
pub type ForkDigest = [u8; 4];
pub type Domain = [u8; 32];

pub type ExecutionAddress = ByteVector<20>;

pub type ChainId = usize;
pub type NetworkId = usize;

pub type RandaoReveal = BlsSignature;
pub type Bytes32 = ByteVector<32>;

pub type ParticipationFlags = u8;

// Coordinate refers to a unique location in the block tree
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coordinate {
    #[serde(with = "crate::serde::as_string")]
    slot: Slot,
    root: Root,
}

pub const GENESIS_SLOT: Slot = 0;
pub const GENESIS_EPOCH: Epoch = 0;
pub const FAR_FUTURE_EPOCH: Epoch = Epoch::MAX;

pub const BLS_WITHDRAWAL_PREFIX: u8 = 0x00;
pub const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn test_serde() {
        let bytes = Bytes32::default();
        let json = serde_json::to_string(&bytes).unwrap();
        assert_eq!(
            json,
            "\"0x0000000000000000000000000000000000000000000000000000000000000000\""
        );
        let bytes_roundtrip: Bytes32 = serde_json::from_str(&json).unwrap();
        assert_eq!(bytes, bytes_roundtrip);
    }
}
