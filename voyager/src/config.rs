use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use voyager_message::context::{
    equivalent_chain_ids::EquivalentChainIds, ModulesConfig, PluginConfig,
};

use crate::queue::QueueConfig;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // allows for using $schema in the config file
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "EquivalentChainIds::is_empty")]
    pub equivalent_chain_ids: EquivalentChainIds,
    pub modules: ModulesConfig,
    pub plugins: Vec<PluginConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoyagerConfig {
    pub num_workers: u16,
    #[serde(default = "default_rest_laddr")]
    pub rest_laddr: SocketAddr,
    #[serde(default = "default_rpc_laddr")]
    pub rpc_laddr: SocketAddr,
    #[serde(default = "default_metrics_endpoint")]
    pub metrics_endpoint: String,
    pub queue: QueueConfig,
    // TODO: Specify per plugin
    #[serde(default = "default_optimizer_delay_milliseconds")]
    pub optimizer_delay_milliseconds: u64,
    #[serde(default = "default_ipc_client_request_timeout")]
    pub ipc_client_request_timeout: Duration,
    pub cache: voyager_message::rpc::server::cache::Config,
}

#[must_use]
#[inline]
pub const fn default_rest_laddr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7177)
}

#[must_use]
#[inline]
pub const fn default_rpc_laddr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7178)
}

#[must_use]
pub fn default_metrics_endpoint() -> String {
    "http://localhost:4318/v1/metrics".to_owned()
}

#[must_use]
#[inline]
pub const fn default_optimizer_delay_milliseconds() -> u64 {
    100
}

#[must_use]
#[inline]
pub const fn default_ipc_client_request_timeout() -> Duration {
    Duration::new(60, 0)
}
