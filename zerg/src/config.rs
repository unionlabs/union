use serde::{Deserialize, Serialize};
use unionlabs::hash::H160;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub evm: chain_utils::evm::Config,
    // TODO: bech32 encoded address
    pub union_contract: String,
    pub evm_contract: H160,
    pub channel: String,
    pub port: String,
    pub rush_blocks: u64,
}
