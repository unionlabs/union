use beacon_api_types::{altair, capella, deneb, electra};
use serde::{Deserialize, Serialize};
use unionlabs::never::Never;

use crate::client::VersionedResponseTypes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightClientBootstrapResponseTypes {}

impl VersionedResponseTypes for LightClientBootstrapResponseTypes {
    type Phase0 = Never;
    type Altair = altair::LightClientBootstrap;
    type Bellatrix = altair::LightClientBootstrap;
    type Capella = capella::LightClientBootstrap;
    type Deneb = deneb::LightClientBootstrap;
    type Electra = electra::LightClientBootstrap;
}
