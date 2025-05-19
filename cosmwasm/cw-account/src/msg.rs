use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
}

#[cw_serde]
pub struct ExecuteMsg {
    pub messages: Vec<CosmosMsg>,
}
