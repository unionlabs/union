use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_types::BitVector;
use tree_hash::TreeHash;

use crate::{
    bls::BlsSignature, errors::InvalidLength, ethereum_consts_traits::SYNC_COMMITTEE_SIZE, Proto,
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct SyncAggregate<C: SYNC_COMMITTEE_SIZE> {
    pub sync_committee_bits: BitVector<C::SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: BlsSignature,
}

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

#[derive(Debug)]
pub enum TryFromSyncAggregateError {
    Bits(ssz_types::Error),
    Signature(InvalidLength),
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
                .map_err(TryFromSyncAggregateError::Bits)?,
            sync_committee_signature: value
                .sync_committee_signature
                .try_into()
                .map_err(TryFromSyncAggregateError::Signature)?,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::SyncAggregate {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.SyncAggregate";
}

impl<C: SYNC_COMMITTEE_SIZE> Proto for SyncAggregate<C> {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::SyncAggregate;
}
