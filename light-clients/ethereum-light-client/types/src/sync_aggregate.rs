use serde::{Deserialize, Serialize};
use ssz::{types::BitVector, Ssz};

use crate::{bls::BlsSignature, errors::InvalidLength, ethereum::config::SYNC_COMMITTEE_SIZE};

impl<C: SYNC_COMMITTEE_SIZE> From<SyncAggregate<C>>
    for protos::union::ibc::lightclients::ethereum::v1::SyncAggregate
{
    fn from(value: SyncAggregate<C>) -> Self {
        Self {
            sync_committee_bits: value.sync_committee_bits.into_bytes().into_vec(),
            sync_committee_signature: value.sync_committee_signature.into_bytes().into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromSyncAggregateError {
    #[error("invalid `sync_committee_bits`")]
    SyncCommitteeBits(#[from] ssz::types::bitfield::BitlistFromBytesError),
    #[error("invalid `sync_committee_signature`")]
    SyncCommitteeSignature(#[from] InvalidLength),
}

impl<C: SYNC_COMMITTEE_SIZE> TryFrom<protos::union::ibc::lightclients::ethereum::v1::SyncAggregate>
    for SyncAggregate<C>
{
    type Error = TryFromSyncAggregateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::SyncAggregate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            // REVIEW: This might not be the correct conversion
            sync_committee_bits: BitVector::from_bytes(value.sync_committee_bits.into())
                .map_err(TryFromSyncAggregateError::SyncCommitteeBits)?,
            sync_committee_signature: value
                .sync_committee_signature
                .try_into()
                .map_err(TryFromSyncAggregateError::SyncCommitteeSignature)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnboundedSyncAggregate {
    #[serde(with = "::serde_utils::hex_string")]
    pub sync_committee_bits: Vec<u8>,
    pub sync_committee_signature: BlsSignature,
}

impl From<UnboundedSyncAggregate>
    for protos::union::ibc::lightclients::ethereum::v1::SyncAggregate
{
    fn from(value: UnboundedSyncAggregate) -> Self {
        Self {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature.into_bytes().into(),
        }
    }
}
