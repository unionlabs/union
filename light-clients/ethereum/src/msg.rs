use cosmwasm_schema::{cw_serde, QueryResponses};

pub use wasm_lc_types::msg::{ExecuteMsg, QueryMsg};

#[cw_serde]
pub struct InstantiateMsg {}
