use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
pub use wasm_light_client_types::msg::{ExecuteMsg, QueryMsg};

#[cw_serde]
pub struct InstantiateMsg {}

pub enum StorageState {
    Occupied(Binary),
    Empty,
}
