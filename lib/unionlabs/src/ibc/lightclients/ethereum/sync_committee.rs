use macros::model;
use ssz::{types::Vector, Ssz};
use typenum::Unsigned;

use crate::{bls::BlsPublicKey, errors::InvalidLength, ethereum::config::SYNC_COMMITTEE_SIZE};

#[derive(Ssz)]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncCommittee),
    into,
    from
))]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    #[serde(with = "::serde_utils::hex_string_list")]
    pub pubkeys: Vector<BlsPublicKey, C::SYNC_COMMITTEE_SIZE>,
    #[serde(with = "::serde_utils::hex_string")]
    pub aggregate_pubkey: BlsPublicKey,
}

impl<C: SYNC_COMMITTEE_SIZE> From<SyncCommittee<C>>
    for protos::union::ibc::lightclients::ethereum::v1::SyncCommittee
{
    fn from(value: SyncCommittee<C>) -> Self {
        Self {
            pubkeys: value.pubkeys.iter().copied().map(Into::into).collect(),
            aggregate_pubkey: value.aggregate_pubkey.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TryFromSyncCommitteeError {
    /// One of the `pubkeys` had an invalid length
    PubKey(InvalidLength),
    /// Invalid amount of `pubkeys`
    PubKeys(InvalidLength),
    /// The `aggregate_pubkey` had an invalid length
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
                .map_err(|vec: Vec<_>| {
                    TryFromSyncCommitteeError::PubKeys(InvalidLength {
                        expected: crate::errors::ExpectedLength::Exact(
                            C::SYNC_COMMITTEE_SIZE::USIZE,
                        ),
                        found: vec.len(),
                    })
                })?,
            aggregate_pubkey: value
                .aggregate_pubkey
                .try_into()
                .map_err(TryFromSyncCommitteeError::AggregatePubKey)?,
        })
    }
}
