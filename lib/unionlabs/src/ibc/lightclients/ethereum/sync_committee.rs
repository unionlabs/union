use macros::model;
use ssz::{types::Vector, Ssz};

use crate::{ethereum::config::SYNC_COMMITTEE_SIZE, hash::H384};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncCommittee),
    into,
    from
))]
#[derive(Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct SyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    pub pubkeys: Vector<H384, C::SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: H384,
}

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::SyncCommittee),
    from
))]
pub struct UnboundedSyncCommittee {
    pub pubkeys: Vec<H384>,
    pub aggregate_pubkey: H384,
}

#[cfg(feature = "proto")]
pub mod proto {
    use typenum::Unsigned;

    use crate::{
        errors::InvalidLength,
        ethereum::config::SYNC_COMMITTEE_SIZE,
        ibc::lightclients::ethereum::sync_committee::{SyncCommittee, UnboundedSyncCommittee},
    };

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

    impl<C: SYNC_COMMITTEE_SIZE>
        TryFrom<protos::union::ibc::lightclients::ethereum::v1::SyncCommittee>
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
}
