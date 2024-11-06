use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct MsgCreateClient {
    pub client_id: u32,
    pub client_state: Binary,
    pub consensus_state: Binary,
}

#[cw_serde]
pub struct MsgUpdateClient {
    pub client_id: u32,
    pub message: Binary,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
}
