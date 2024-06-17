use std::collections::HashMap;

use chain_utils::private_key::PrivateKey;
use ethers::core::k256::ecdsa;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{ChainSpec, PresetBaseKind},
    hash::H160,
    id::ChannelId,
};

use crate::chains::Protocol;

pub const KEY_ETHEREUM: &str = "ethereum";
pub const KEY_OSMOSIS: &str = "osmosis";
pub const KEY_UNION: &str = "union";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chain_configs: HashMap<String, AnyChainConfig>,
    pub interactions: Vec<IbcInteraction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub ibc_handler_address: H160,
    pub eth_rpc_api: String,
    pub transfer_module: TransferModule,
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CosmosConfig {
    pub chain_config: chain_utils::cosmos::Config,
    pub transfer_module: TransferModule,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferModule {
    Native,
    Contract { address: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IbcInteraction {
    pub source: Endpoint,
    pub destination: Endpoint,
    pub send_packet_interval: u64,
    pub expect_full_cycle: u64,
    pub protocol: Protocol,
    pub amount_min: u64,
    pub amount_max: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub chain: String,
    pub channel: ChannelId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnyChainConfig {
    Cosmos(CosmosConfig),
    Ethereum(EthereumConfig),
}
