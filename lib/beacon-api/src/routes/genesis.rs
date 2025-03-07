use beacon_api_types::custom_types::Version;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisResponse {
    pub data: GenesisData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisData {
    pub genesis_validators_root: H256,
    #[serde(with = "::serde_utils::string")]
    pub genesis_time: u64,
    pub genesis_fork_version: Version,
}
