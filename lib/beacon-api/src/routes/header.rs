use beacon_api_types::phase0;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::H256;

use crate::client::{VersionedResponse, VersionedResponseTypes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockHeaderResponse {
    #[serde(flatten)]
    pub response: VersionedResponse<BeaconBlockHeaderResponseTypes>,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconHeaderData<H> {
    pub root: H256,
    pub canonical: bool,
    pub header: H,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeaconBlockHeaderResponseTypes {}

impl VersionedResponseTypes for BeaconBlockHeaderResponseTypes {
    type Phase0 = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
    type Altair = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
    type Bellatrix = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
    type Capella = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
    type Deneb = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
    type Electra = BeaconHeaderData<phase0::SignedBeaconBlockHeader>;
}
