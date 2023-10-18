use serde::{Deserialize, Serialize};
use unionlabs::ethereum::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub evm: chain_utils::evm::Config,
    pub union_contract: String,
    pub evm_contract: Address,
    pub channel: String,
    pub rush_blocks: u64,
}
