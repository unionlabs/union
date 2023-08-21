use cosmwasm_schema::cw_serde;
pub use wasm_light_client_types::msg::{ExecuteMsg, QueryMsg};

#[cw_serde]
pub struct InstantiateMsg {}

pub enum StorageState {
    Occupied(Vec<u8>),
    Empty,
}
