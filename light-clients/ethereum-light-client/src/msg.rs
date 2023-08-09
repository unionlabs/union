use cosmwasm_schema::cw_serde;
use serde::{Deserialize, Serialize};
use unionlabs::ethereum_consts_traits::PresetBaseKind;
pub use wasm_light_client_types::msg::ExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum QueryMsg {
    /// Returns the ethereum preset that this contract is compiled for
    EthPreset {},
    // / Queries that are defined in the wasm client specification
    #[serde(untagged)]
    LightClientSpecification(wasm_light_client_types::msg::QueryMsg),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthPresetResponse {
    pub preset: PresetBaseKind,
}
