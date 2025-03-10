use beacon_api_types::{altair, capella, deneb, electra};
use serde::{Deserialize, Serialize};
use unionlabs::never::Never;

use crate::client::VersionedResponseTypes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightClientUpdateResponseTypes {}

impl VersionedResponseTypes for LightClientUpdateResponseTypes {
    type Phase0 = Never;
    type Altair = altair::LightClientUpdate;
    type Bellatrix = altair::LightClientUpdate;
    type Capella = capella::LightClientUpdate;
    type Deneb = deneb::LightClientUpdate;
    type Electra = electra::LightClientUpdate;
}
