use core::fmt::Debug;

use serde::{Deserialize, Serialize};

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
    sync_aggregate::UnboundedSyncAggregate,
};

pub type NextSyncCommitteeBranch = [H256; floorlog2(NEXT_SYNC_COMMITTEE_INDEX)];
pub type FinalityBranch = [H256; floorlog2(FINALIZED_ROOT_INDEX)];

// fn try_from_proto_branch<T>(proto: Vec<Vec<u8>>) -> Result<T, TryFromBranchError<T>>
// where
//     T: TryFrom<Vec<H256>, Error: Debug + PartialEq + Eq + Clone>,
// {
//     proto
//         .into_iter()
//         .map(H256::try_from)
//         .collect::<Result<Vec<_>, _>>()
//         .map_err(TryFromBranchError::BranchNode)?
//         .try_into()
//         .map_err(TryFromBranchError::Branch)
// }

// // TODO: Remove the bounds on T::Error and only require said bounds when implementing the respective traits, will clean up try_from_proto_branch as well
// #[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
// pub enum TryFromBranchError<T>
// where
//     T: TryFrom<Vec<H256>, Error: Debug + PartialEq + Eq + Clone>,
// {
//     #[error("error decoding branch: {0:?}")]
//     Branch(<T as TryFrom<Vec<H256>>>::Error),
//     #[error("error decoding branch node")]
//     BranchNode(#[source] InvalidLength),
// }

pub enum LightClientUpdate {
    EpochChange {
        /// The trusted sync committee for the *next* epoch that the client is being updated to.
        ///
        /// If the current epoch is 10, this will be the sync committee for epoch 11.
        sync_committee: SyncCommittee,

        /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
        ///
        /// If the current epoch is 10, this will be the next sync committee for epoch 11 (i.e. the sync committee for epoch 12).
        next_sync_committee: SyncCommittee,
        /// The path of the next sync committee in the beacon SSZ state root.
        next_sync_committee_branch: NextSyncCommitteeBranch,

        update_data: LightClientUpdateData,
    },
    WithinEpoch {
        /// The trusted sync committee for the current epoch.
        ///
        /// If the current epoch is 10, this will be the sync committee for epoch 10.
        sync_committee: SyncCommittee<C>,

        update_data: LightClientUpdateData,
    },
}

/// Common data required for all light client updates.
pub struct LightClientUpdateData {
    /// Header attested to by the sync committee
    pub attested_header: UnboundedLightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: UnboundedLightClientHeader,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: UnboundedSyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}
