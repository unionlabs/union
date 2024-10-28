use unionlabs::hash::H256;

use crate::{
    consts::{floorlog2, FINALIZED_ROOT_INDEX},
    light_client_header::LightClientHeader,
    SyncAggregate,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientFinalityUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub signature_slot: u64,
}
