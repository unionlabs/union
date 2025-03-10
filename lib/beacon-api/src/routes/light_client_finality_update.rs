use beacon_api_types::{altair, bellatrix, capella, deneb, electra};
use serde::{Deserialize, Serialize};
use unionlabs::never::Never;

use crate::client::VersionedResponseTypes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightClientFinalityUpdateResponseTypes {}

impl VersionedResponseTypes for LightClientFinalityUpdateResponseTypes {
    type Phase0 = Never;
    type Altair = altair::LightClientFinalityUpdate;
    type Bellatrix = bellatrix::LightClientFinalityUpdate;
    type Capella = capella::LightClientFinalityUpdate;
    type Deneb = deneb::LightClientFinalityUpdate;
    type Electra = electra::LightClientFinalityUpdate;
}
