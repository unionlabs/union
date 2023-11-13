use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_types::{fixed_vector, FixedVector};
use tree_hash::TreeHash;

use crate::{
    bls::BlsPublicKey, errors::InvalidLength, ethereum::config::SYNC_COMMITTEE_SIZE, Proto, TypeUrl,
};

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    #[serde(with = "::serde_utils::hex_string_list")]
    pub pubkeys: FixedVector<BlsPublicKey, C::SYNC_COMMITTEE_SIZE>,
    #[serde(with = "::serde_utils::hex_string")]
    pub aggregate_pubkey: BlsPublicKey,
}

impl<C: SYNC_COMMITTEE_SIZE> From<SyncCommittee<C>>
    for protos::union::ibc::lightclients::ethereum::v1::SyncCommittee
{
    fn from(value: SyncCommittee<C>) -> Self {
        Self {
            pubkeys: value.pubkeys.iter().cloned().map(Into::into).collect(),
            aggregate_pubkey: value.aggregate_pubkey.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromSyncCommitteeError {
    /// PubKey had an invalid length
    PubKey(InvalidLength),
    /// Invalid amount of pubkeys
    PubKeys(fixed_vector::TryFromVecError),
    /// AggregatePubKey had an invalid length
    AggregatePubKey(InvalidLength),
}

impl<C: SYNC_COMMITTEE_SIZE> TryFrom<protos::union::ibc::lightclients::ethereum::v1::SyncCommittee>
    for SyncCommittee<C>
{
    type Error = TryFromSyncCommitteeError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::SyncCommittee,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            pubkeys: value
                .pubkeys
                .iter()
                .cloned()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromSyncCommitteeError::PubKey)?
                .try_into()
                .map_err(TryFromSyncCommitteeError::PubKeys)?,
            aggregate_pubkey: value
                .aggregate_pubkey
                .try_into()
                .map_err(TryFromSyncCommitteeError::AggregatePubKey)?,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::SyncCommittee {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.SyncCommittee";
}

impl<C: SYNC_COMMITTEE_SIZE> Proto for SyncCommittee<C> {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::SyncCommittee;
}
