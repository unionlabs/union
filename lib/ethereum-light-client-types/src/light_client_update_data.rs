use beacon_api_types::{
    light_client_update::{FinalityBranch, NextSyncCommitteeBranch},
    LightClientHeader, SyncAggregate, SyncCommittee,
};

/// Common data required for all light client updates.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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

impl LightClientUpdateData {
    pub fn into_beacon_light_client_update(
        self,
        next_sync_committee: Option<SyncCommittee>,
        next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    ) -> beacon_api_types::LightClientUpdate {
        beacon_api_types::LightClientUpdate {
            attested_header: self.attested_header,
            next_sync_committee,
            next_sync_committee_branch,
            finalized_header: self.finalized_header,
            finality_branch: self.finality_branch,
            sync_aggregate: self.sync_aggregate,
            signature_slot: self.signature_slot,
        }
    }
}
