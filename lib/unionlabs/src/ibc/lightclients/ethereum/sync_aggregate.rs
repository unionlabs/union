use macros::model;
use ssz::{types::BitVector, TreeHash};

use crate::{bls::BlsSignature, errors::InvalidLength, ethereum::config::SYNC_COMMITTEE_SIZE};

#[derive(ssz::Encode, ssz::Decode, TreeHash)]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncAggregate),
    into,
    from
))]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SyncAggregate<C: SYNC_COMMITTEE_SIZE> {
    // TODO: Change debug print for this type in ssz::types
    #[debug("BitVector({})", sync_committee_bits.iter().map(|b| if b { '1' } else { '0' }).collect::<String>())]
    pub sync_committee_bits: BitVector<C::SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: BlsSignature,
}

// pub fn bit_vector_debug<N: Unsigned + Clone>(
//     bv: &BitVector<N>,
//     f: &mut fmt::Formatter,
// ) -> fmt::Result {
//     for b in bv.iter() {
//         write!(f, "BitVector({})", if b { '1' } else { '0' })?;
//     }

//     Ok(())
// }

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
    Bits(ssz::types::Error),
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
