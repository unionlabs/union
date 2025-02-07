use cosmwasm_std::Addr;

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
pub struct InitMsg {
    pub ibc_host: Addr,
}

pub type QueryMsg = ibc_union_msg::lightclient::QueryMsg;
