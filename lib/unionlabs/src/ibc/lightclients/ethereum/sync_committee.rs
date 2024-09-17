use macros::model;
use ssz::{types::Vector, Ssz};
use typenum::Unsigned;

use crate::{bls::BlsPublicKey, errors::InvalidLength, ethereum::config::SYNC_COMMITTEE_SIZE};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncCommittee),
    into,
    from
))]
#[derive(Ssz)]
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

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromSyncCommitteeError {
    #[error("invalid `pubkeys`")]
    PubKey(#[source] InvalidLength),
    #[error("invalid amount of `pubkeys`")]
    PubKeys(#[source] InvalidLength),
    #[error("invalid `aggregate_pubkey`")]
    AggregatePubKey(#[source] InvalidLength),
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

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncCommittee),
    from
))]
pub struct UnboundedSyncCommittee {
    #[serde(with = "::serde_utils::hex_string_list")]
    pub pubkeys: Vec<BlsPublicKey>,
    #[serde(with = "::serde_utils::hex_string")]
    pub aggregate_pubkey: BlsPublicKey,
}

impl From<UnboundedSyncCommittee>
    for protos::union::ibc::lightclients::ethereum::v1::SyncCommittee
{
    fn from(value: UnboundedSyncCommittee) -> Self {
        Self {
            pubkeys: value.pubkeys.iter().copied().map(Into::into).collect(),
            aggregate_pubkey: value.aggregate_pubkey.into(),
        }
    }
}
