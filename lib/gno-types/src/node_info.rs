use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, encoding::Base64};

use crate::{NodeInfoOther, VersionInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version_set: Vec<VersionInfo>,
    /// Example: `g129twzse9fks86wsc85967cn6rw905lxyz9463u@0.0.0.0:26656`
    pub net_address: String,
    pub network: String,
    pub version: String,
    pub channels: Bytes<Base64>,
    pub moniker: String,
    pub other: NodeInfoOther,
}
