use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use voyager_message::context::{ModulesConfig, PluginConfig};

use crate::queue::QueueConfig;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // allows for using $schema in the config file
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
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
    pub queue: QueueConfig,
    // TODO: Specify per plugin
    #[serde(default = "default_optimizer_delay_milliseconds")]
    pub optimizer_delay_milliseconds: u64,
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
#[inline]
pub const fn default_optimizer_delay_milliseconds() -> u64 {
    100
}
