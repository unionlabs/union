use beacon_api_types::{
    altair::{SyncAggregate, SyncCommittee},
    custom_types::Slot,
    deneb, electra,
};
use unionlabs_primitives::H256;

use crate::LightClientHeader;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LightClientUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<H256>>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<H256>,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: Slot,
}

impl From<deneb::LightClientUpdate> for LightClientUpdate {
    fn from(value: deneb::LightClientUpdate) -> Self {
        Self {
            attested_header: value.attested_header.into(),
            next_sync_committee: Some(value.next_sync_committee),
            next_sync_committee_branch: Some(value.next_sync_committee_branch.to_vec()),
            finalized_header: value.finalized_header.into(),
            finality_branch: value.finality_branch.to_vec(),
            sync_aggregate: value.sync_aggregate,
            signature_slot: value.signature_slot,
        }
    }
}

impl From<electra::LightClientUpdate> for LightClientUpdate {
    fn from(value: electra::LightClientUpdate) -> Self {
        Self {
            attested_header: value.attested_header.into(),
            next_sync_committee: Some(value.next_sync_committee),
            next_sync_committee_branch: Some(value.next_sync_committee_branch.to_vec()),
            finalized_header: value.finalized_header.into(),
            finality_branch: value.finality_branch.to_vec(),
            sync_aggregate: value.sync_aggregate,
            signature_slot: value.signature_slot,
        }
    }
}
