use serde::{ Deserialize, Serialize };
use unionlabs::hash::H160;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub ethereum: chain_utils::ethereum::Config,
    pub osmosis: chain_utils::cosmos::Config,
    pub union_contract: String,
    pub osmosis_contract: String,
    pub channel: String,
    pub port: String,
    pub amount: String,
    pub rush_blocks: u64,
    pub datadog_data: DatadogData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatadogData {
    pub datadog_api_key: String,
    pub datadog_log_host: String,
    pub datadog_validate_host: String,
}
