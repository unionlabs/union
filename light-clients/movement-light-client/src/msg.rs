use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub ibc_host: Addr,
}

pub type QueryMsg = union_ibc::lightclient::query::QueryMsg;
