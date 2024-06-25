use std::collections::HashMap;

use chain_utils::private_key::PrivateKey;
use chrono::{DateTime, Utc};
use ethers::{core::k256::ecdsa, types::H256};
use serde::{Deserialize, Serialize};
use unionlabs::{hash::H160, id::ChannelId};

use crate::chains::Protocol;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub chain_configs: HashMap<String, AnyChainConfig>,
    pub interactions: Vec<IbcInteraction>,
    pub single_interaction: Option<IbcInteraction>, // This is just to send single transaction and close the program
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventTrackerConfig {
    pub idx: u64,      // 0: sendpacket, 1: recvpacket, 2:writeack, 3:acknowledge
    pub arrived: bool, // is packet arrived or not
    pub arrived_time: Option<DateTime<Utc>>, // time when packet arrived
    pub tx_hash: Option<H256>, // hash of the transaction
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthereumConfig {
    pub ibc_handler_address: H160,
    pub eth_rpc_api: String,
    pub transfer_module: TransferModule,
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CosmosConfig {
    pub chain_config: chain_utils::cosmos::Config,
    pub transfer_module: TransferModule,
    pub enabled: bool,
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
    pub memo: String,
    pub sending_memo_probability: f64,
    pub denoms: Vec<String>,
    pub max_retry: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub chain: String,
    pub channel: ChannelId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AnyChainConfig {
    Cosmos(CosmosConfig),
    Ethereum(EthereumConfig),
}
