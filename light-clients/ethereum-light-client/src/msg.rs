use cosmwasm_schema::cw_serde;
use serde::{Deserialize, Serialize};
use unionlabs::ethereum_consts_traits::PresetBaseKind;
pub use wasm_light_client_types::msg::ExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum QueryMsg {
    /// Queries that are defined in the wasm client specification
    LightClientSpecification(wasm_light_client_types::msg::QueryMsg),
    /// Returns the ethereum preset that this contract is compiled for
    EthPreset {},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthPresetResponse {
    pub preset: PresetBaseKind,
}
