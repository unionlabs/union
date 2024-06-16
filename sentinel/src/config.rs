use serde::{ Deserialize, Serialize };
use unionlabs::{ ethereum::config::{ ChainSpec, PresetBaseKind }, hash::H160, id::ChannelId };

use crate::chains::Protocol;

pub const KEY_ETHEREUM: &str = "ethereum";
pub const KEY_OSMOSIS: &str = "osmosis";
pub const KEY_UNION: &str = "union";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ethereum: EthereumConfig,
    pub osmosis: CosmosConfig,
    pub union: CosmosConfig,
    pub interactions: Vec<IbcInteraction>,
    pub db_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub enable: bool,
    pub preset: PresetBaseKind,
    pub chain_config: chain_utils::ethereum::Config,
    pub transfer_module: TransferModule,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CosmosConfig {
    pub enable: bool,
    pub chain_config: chain_utils::cosmos::Config,
    pub transfer_module: TransferModule,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferModule {
    Native,
    Contract {
        address: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IbcInteraction {
    pub source: Endpoint,
    pub destination: Endpoint,
    pub send_packet_interval: u64,
    pub expect_full_cycle: u64,
    pub protocol: Protocol,
    pub amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub chain: String,
    pub channel: ChannelId,
}
