use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use voyager_message::context::PluginConfig;

use crate::queue::AnyQueueConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub plugins: Vec<PluginConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoyagerConfig {
    pub num_workers: u16,
    pub laddr: SocketAddr,
    pub queue: AnyQueueConfig,
    // pub tx_batch: TxBatch,
    #[serde(default)]
    pub optimizer_delay_milliseconds: u64,
}
