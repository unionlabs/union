use beacon_api_types::{light_client_update::NextSyncCommitteeBranch, SyncCommittee};

use crate::LightClientUpdateData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// boxed for size
pub enum LightClientUpdate {
    EpochChange(Box<EpochChangeUpdate>),
    WithinEpoch(Box<WithinEpochUpdate>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct EpochChangeUpdate {
    /// The trusted sync committee for the *next* epoch that the client is being updated to.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 11.
    pub sync_committee: SyncCommittee,

    /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
    ///
    /// If the current epoch is 10, this will be the *next* sync committee for epoch 11 (i.e. the sync committee for epoch 12).
    pub next_sync_committee: SyncCommittee,
    /// The path of the next sync committee in the beacon chain SSZ state root.
    pub next_sync_committee_branch: NextSyncCommitteeBranch,

    pub update_data: LightClientUpdateData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct WithinEpochUpdate {
    /// The trusted sync committee for the *current* epoch.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 10.
    pub sync_committee: SyncCommittee,

    pub update_data: LightClientUpdateData,
}

impl LightClientUpdate {
    pub fn update_data(&self) -> &LightClientUpdateData {
        match self {
            LightClientUpdate::EpochChange(update) => &update.update_data,
            LightClientUpdate::WithinEpoch(update) => &update.update_data,
        }
    }

    /// `ethereum-sync-protocol` takes both `current_sync_committee` and `next_sync_committee` as a parameter.
    /// Although theoretically it can work when both params to be `Some`, for optimization reasons, the client
    /// will only pass one at a time based on the update type. This function returns the currently trusted sync committee
    /// in tuple format ready to be passed in to the verifier.
    ///
    /// Returns `(current_sync_committee, next_sync_committee)`
    pub fn currently_trusted_sync_committee(
        &self,
    ) -> (Option<&SyncCommittee>, Option<&SyncCommittee>) {
        match self {
            LightClientUpdate::EpochChange(update) => (None, Some(&update.sync_committee)),
            LightClientUpdate::WithinEpoch(update) => (Some(&update.sync_committee), None),
        }
    }
}

impl From<LightClientUpdate> for beacon_api_types::LightClientUpdate {
    fn from(value: LightClientUpdate) -> Self {
        match value {
            LightClientUpdate::EpochChange(update) => {
                update.update_data.into_beacon_light_client_update(
                    Some(update.next_sync_committee),
                    Some(update.next_sync_committee_branch),
                )
            }
            LightClientUpdate::WithinEpoch(update) => update
                .update_data
                .into_beacon_light_client_update(None, None),
        }
    }
}
