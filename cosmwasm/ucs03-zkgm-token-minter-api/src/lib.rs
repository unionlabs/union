use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use enumorph::Enumorph;
use token_factory_api::TokenFactoryMsg;

#[cw_serde]
pub enum LocalTokenMsg {
    TakeFunds {
        from: String,
        denom: String,
        recipient: String,
        amount: Uint128,
    },
    Transfer {
        denom: String,
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(Enumorph)]
pub enum ExecuteMsg {
    Wrapped(TokenFactoryMsg),
    Local(LocalTokenMsg),
}

pub type QueryMsg = token_factory_api::TokenFactoryQuery;
