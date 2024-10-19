use core::fmt::Debug;

use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    ethereum::config::{
        consts::{floorlog2, FINALIZED_ROOT_INDEX, NEXT_SYNC_COMMITTEE_INDEX},
        BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE,
    },
    hash::H256,
    ibc::lightclients::ethereum::{
        light_client_header::{
            LightClientHeader, TryFromLightClientHeaderError, UnboundedLightClientHeader,
        },
        sync_aggregate::{SyncAggregate, TryFromSyncAggregateError, UnboundedSyncAggregate},
        sync_committee::{SyncCommittee, TryFromSyncCommitteeError, UnboundedSyncCommittee},
    },
};

/// TODO: Move these to a more central location
pub type NextSyncCommitteeBranch = [H256; floorlog2(NEXT_SYNC_COMMITTEE_INDEX)];
pub type FinalityBranch = [H256; floorlog2(FINALIZED_ROOT_INDEX)];

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate),
    into,
    from
))]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientUpdate<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader<C>,
    /// Next sync committee corresponding to `attested_header.state_root`
    // NOTE: These fields aren't actually optional, they are just because of the current structure of the ethereum Header.
    // TODO: Remove the Option and improve ethereum::header::Header to be an enum, instead of using optional fields and bools.
    #[serde(default)]
    pub next_sync_committee: Option<SyncCommittee<C>>,
    #[serde(default)]
    pub next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader<C>,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate<C>,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
    From<LightClientUpdate<C>>
    for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate
{
    fn from(value: LightClientUpdate<C>) -> Self {
        Self {
            attested_header: Some(value.attested_header.into()),
            next_sync_committee: value.next_sync_committee.map(Into::into),
            next_sync_committee_branch: value
                .next_sync_committee_branch
                .unwrap_or_default()
                .iter()
                .copied()
                .map(H256::into_bytes)
                .collect(),
            finalized_header: Some(value.finalized_header.into()),
            finality_branch: value
                .finality_branch
                .iter()
                .copied()
                .map(H256::into_bytes)
                .collect(),
            sync_aggregate: Some(value.sync_aggregate.into()),
            signature_slot: value.signature_slot,
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromLightClientUpdateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid `attested_header`")]
    AttestedHeader(#[source] TryFromLightClientHeaderError),
    #[error("invalid `next_sync_committee`")]
    NextSyncCommittee(#[from] TryFromSyncCommitteeError),
    #[error("invalid `next_sync_committee_branch`")]
    NextSyncCommitteeBranch(#[from] TryFromBranchError<NextSyncCommitteeBranch>),
    #[error("invalid `finality_branch`")]
    FinalityBranch(#[from] TryFromBranchError<FinalityBranch>),
    #[error("invalid `sync_aggregate`")]
    SyncAggregate(#[from] TryFromSyncAggregateError),
    #[error("invalid `finalized_header`")]
    FinalizedHeader(#[source] TryFromLightClientHeaderError),
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate>
    for LightClientUpdate<C>
{
    type Error = TryFromLightClientUpdateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            attested_header: required!(value.attested_header)?
                .try_into()
                .map_err(TryFromLightClientUpdateError::AttestedHeader)?,
            next_sync_committee: value
                .next_sync_committee
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromLightClientUpdateError::NextSyncCommittee)?,
            next_sync_committee_branch: if value.next_sync_committee_branch.is_empty() {
                None
            } else {
                Some(
                    try_from_proto_branch(value.next_sync_committee_branch)
                        .map_err(TryFromLightClientUpdateError::NextSyncCommitteeBranch)?,
                )
            },
            finalized_header: required!(value.finalized_header)?
                .try_into()
                .map_err(TryFromLightClientUpdateError::FinalizedHeader)?,
            finality_branch: try_from_proto_branch(value.finality_branch)
                .map_err(TryFromLightClientUpdateError::FinalityBranch)?,
            sync_aggregate: required!(value.sync_aggregate)?
                .try_into()
                .map_err(TryFromLightClientUpdateError::SyncAggregate)?,
            signature_slot: value.signature_slot,
        })
    }
}

fn try_from_proto_branch<T>(proto: Vec<Vec<u8>>) -> Result<T, TryFromBranchError<T>>
where
    T: TryFrom<Vec<H256>, Error: Debug + PartialEq + Eq + Clone>,
{
    proto
        .into_iter()
        .map(H256::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(TryFromBranchError::BranchNode)?
        .try_into()
        .map_err(TryFromBranchError::Branch)
}

// TODO: Remove the bounds on T::Error and only require said bounds when implementing the respective traits, will clean up try_from_proto_branch as well
#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum TryFromBranchError<T>
where
    T: TryFrom<Vec<H256>, Error: Debug + PartialEq + Eq + Clone>,
{
    #[error("error decoding branch: {0:?}")]
    Branch(<T as TryFrom<Vec<H256>>>::Error),
    #[error("error decoding branch node")]
    BranchNode(#[source] InvalidLength),
}

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate),
    from
))]
pub struct UnboundedLightClientUpdate {
    /// Header attested to by the sync committee
    pub attested_header: UnboundedLightClientHeader,
    /// Next sync committee corresponding to `attested_header.state_root`
    // NOTE: These fields aren't actually optional, they are just because of the current structure of the ethereum Header.
    // TODO: Remove the Option and improve ethereum::header::Header to be an enum, instead of using optional fields and bools.
    #[serde(default)]
    pub next_sync_committee: Option<UnboundedSyncCommittee>,
    #[serde(default)]
    pub next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: UnboundedLightClientHeader,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: UnboundedSyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}

impl From<UnboundedLightClientUpdate>
    for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate
{
    fn from(value: UnboundedLightClientUpdate) -> Self {
        Self {
            attested_header: Some(value.attested_header.into()),
            next_sync_committee: value.next_sync_committee.map(Into::into),
            next_sync_committee_branch: value
                .next_sync_committee_branch
                .unwrap_or_default()
                .iter()
                .copied()
                .map(H256::into_bytes)
                .collect(),
            finalized_header: Some(value.finalized_header.into()),
            finality_branch: value
                .finality_branch
                .iter()
                .copied()
                .map(H256::into_bytes)
                .collect(),
            sync_aggregate: Some(value.sync_aggregate.into()),
            signature_slot: value.signature_slot,
        }
    }
}
