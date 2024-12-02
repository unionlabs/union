use beacon_api_types::SyncCommittee;
use unionlabs::ibc::core::client::height::Height;

use crate::LightClientUpdate;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Misbehaviour {
    pub sync_committee: SyncCommittee,
    pub trusted_height: Height,
    pub update_1: LightClientUpdate,
    pub update_2: LightClientUpdate,
}
