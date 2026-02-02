use std::{net::SocketAddr, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use voyager_core::{
    context::{ModulesConfig, PluginConfig},
    default_ipc_client_request_timeout, default_optimizer_delay_milliseconds, default_rest_laddr,
    default_rpc_laddr, default_trace_ratio,
    equivalent_chain_ids::EquivalentChainIds,
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
    #[serde(default = "default_trace_ratio")]
    pub trace_ratio: Option<f64>,
    pub queue: QueueConfig,
    // TODO: Specify per plugin
    #[serde(default = "default_optimizer_delay_milliseconds")]
    pub optimizer_delay_milliseconds: u64,
    #[serde(default = "default_ipc_client_request_timeout")]
    pub ipc_client_request_timeout: Duration,
    pub cache: voyager_core::cache::Config,
}
