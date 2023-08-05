use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use tree_hash::TreeHash;

use crate::{
    errors::MissingField,
    ethereum_consts_traits::SYNC_COMMITTEE_SIZE,
    ibc::{core::client::height::Height, lightclients::ethereum::sync_committee::SyncCommittee},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct TrustedSyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    pub trusted_height: Height,
    pub sync_committee: SyncCommittee<C>,
    pub is_next: bool,
}

impl<C: SYNC_COMMITTEE_SIZE> From<TrustedSyncCommittee<C>>
    for protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee
{
    fn from(value: TrustedSyncCommittee<C>) -> Self {
        Self {
            trusted_height: Some(value.trusted_height.into()),
            sync_committee: Some(value.sync_committee.into()),
            is_next: value.is_next,
        }
    }
}

#[derive(Debug)]
pub enum TryFromTrustedSyncCommitteeError<C: SYNC_COMMITTEE_SIZE> {
    MissingField(MissingField),
    SyncCommittee(TryFromProtoErrorOf<SyncCommittee<C>>),
}

impl<C: SYNC_COMMITTEE_SIZE>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee>
    for TrustedSyncCommittee<C>
{
    type Error = TryFromTrustedSyncCommitteeError<C>;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            trusted_height: value
                .trusted_height
                .ok_or(TryFromTrustedSyncCommitteeError::MissingField(
                    MissingField("trusted_height"),
                ))?
                .into(),
            sync_committee: value
                .sync_committee
                .ok_or(TryFromTrustedSyncCommitteeError::MissingField(
                    MissingField("sync_committee"),
                ))?
                .try_into()
                .map_err(TryFromTrustedSyncCommitteeError::SyncCommittee)?,
            is_next: value.is_next,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.TrustedSyncCommittee";
}

impl<C: SYNC_COMMITTEE_SIZE> Proto for TrustedSyncCommittee<C> {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee;
}
