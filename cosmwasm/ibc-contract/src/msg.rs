use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InitMsg {}

#[cw_serde]
pub struct IbcInitMsg {
    pub client_id: String,
    pub client_state: Binary,
    pub consensus_state: Binary,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterClient {
        code_id: u64,
        client_type: String,
    },

    CreateClient {
        client_type: String,
        client_state: Binary,
        consensus_state: Binary,
    },
}
