use unionlabs::primitives::H768;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SyncAggregate {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub sync_committee_bits: Vec<u8>,
    pub sync_committee_signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ::ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SyncAggregateSsz<C: crate::chain_spec::SYNC_COMMITTEE_SIZE> {
    // TODO: Change debug print for this type in ssz::types
    // #[debug("BitVector({})", sync_committee_bits.iter().map(|b| if b { '1' } else { '0' }).collect::<String>())]
    pub sync_committee_bits: ::ssz::types::BitVector<C::SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: H768,
}

#[cfg(feature = "ssz")]
pub mod ssz {
    use ::ssz::types::{bitfield::BitlistFromBytesError, BitVector};

    use super::*;

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("invalid sync committee bits")]
        SyncCommitteeBits(#[from] BitlistFromBytesError),
    }

    impl<C: crate::chain_spec::SYNC_COMMITTEE_SIZE> TryFrom<SyncAggregate> for SyncAggregateSsz<C> {
        type Error = Error;

        fn try_from(value: SyncAggregate) -> Result<Self, Self::Error> {
            Ok(Self {
                sync_committee_bits: BitVector::from_bytes(value.sync_committee_bits.into())?,
                sync_committee_signature: value.sync_committee_signature,
            })
        }
    }
}
