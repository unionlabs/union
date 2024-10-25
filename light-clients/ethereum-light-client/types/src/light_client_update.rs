use beacon_api_types::{
    light_client_update::{FinalityBranch, NextSyncCommitteeBranch},
    LightClientHeader, SyncAggregate, SyncCommittee,
};
use serde::{Deserialize, Serialize};

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
        sync_committee: SyncCommittee,

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

#[cfg(feature = "proto")]
pub mod proto {}
