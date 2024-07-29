use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use voyager_message::context::ModuleConfig;

use crate::queue::AnyQueueConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub plugins: Vec<ModuleConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoyagerConfig {
    pub num_workers: u16,
    #[serde(default = "default_rest_laddr")]
    pub rest_laddr: SocketAddr,
    #[serde(default = "default_rpc_laddr")]
    pub rpc_laddr: SocketAddr,
    pub queue: AnyQueueConfig,
    // pub tx_batch: TxBatch,
    #[serde(default)]
    pub optimizer_delay_milliseconds: u64,
}

pub fn default_rest_laddr() -> SocketAddr {
    "0.0.0.0:7177".parse().unwrap()
}

pub fn default_rpc_laddr() -> SocketAddr {
    "0.0.0.0:7178".parse().unwrap()
}
