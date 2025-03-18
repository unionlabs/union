use beacon_api_types::phase0;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockHeaderResponse {
    pub data: BeaconHeaderData,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconHeaderData {
    pub root: H256,
    pub canonical: bool,
    pub header: phase0::SignedBeaconBlockHeader,
}
