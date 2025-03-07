use beacon_api_types::{bellatrix, capella, deneb, electra, phase0};
use serde::{Deserialize, Serialize};
use unionlabs::primitives::H256;

use crate::client::{VersionedResponse, VersionedResponseTypes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockResponse {
    #[serde(flatten)]
    pub response: VersionedResponse<BeaconBlockResponseTypes>,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockData<H> {
    pub root: H256,
    pub canonical: bool,
    pub header: H,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeaconBlockResponseTypes {}

impl VersionedResponseTypes for BeaconBlockResponseTypes {
    type Phase0 = phase0::SignedBeaconBlock;
    type Altair = phase0::SignedBeaconBlock;
    type Bellatrix = bellatrix::SignedBeaconBlock;
    type Capella = capella::SignedBeaconBlock;
    type Deneb = deneb::SignedBeaconBlock;
    type Electra = electra::SignedBeaconBlock;
}
