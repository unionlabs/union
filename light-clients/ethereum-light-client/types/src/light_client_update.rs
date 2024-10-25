use beacon_api_types::{LightClientHeader, SyncAggregate};
use serde::{Deserialize, Serialize};

pub type NextSyncCommitteeBranch = [H256; floorlog2(NEXT_SYNC_COMMITTEE_INDEX)];
pub type FinalityBranch = [H256; floorlog2(FINALIZED_ROOT_INDEX)];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        /// The trusted sync committee for the *current* epoch.
        ///
        /// If the current epoch is 10, this will be the sync committee for epoch 10.
        sync_committee: SyncCommittee<C>,

        update_data: LightClientUpdateData,
    },
}

/// Common data required for all light client updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightClientUpdateData {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: u64,
}
