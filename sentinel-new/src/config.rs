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
    pub ethereum: EthereumConfig,
    pub osmosis: OsmosisConfig,
    pub union: UnionConfig,
    pub interactions: Vec<IbcInteraction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub enable: bool,
    pub preset: PresetBaseKind,
    pub chain_config: chain_utils::ethereum::Config,
    pub transfer_module: TransferModule<H160>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsmosisConfig {
    pub enable: bool,
    pub chain_config: chain_utils::cosmos::Config,
    pub transfer_module: TransferModule<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnionConfig {
    pub enable: bool,
    pub chain_config: chain_utils::union::Config,
    pub transfer_module: TransferModule<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TransferModule {
    Native,
    Contract { address: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IbcInteraction {
    pub source: Endpoint,
    pub destination: Endpoint,
    pub send_packet_interval: u64,
    pub expect_full_cycle: u64,
    pub protocol: Protocol,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub chain: String,
    pub channel: ChannelId,
}
