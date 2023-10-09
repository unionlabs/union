use std::collections::BTreeMap;

use chain_utils::private_key::PrivateKey;
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::ethereum::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub evm: chain_utils::evm::Config,
    pub contract: String,
    pub channel: String,
}
