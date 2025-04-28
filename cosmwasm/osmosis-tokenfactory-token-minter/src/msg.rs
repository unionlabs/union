use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg};
use enumorph::Enumorph;
use token_factory_api::{BurnTokensMsg, ChangeAdminMsg, MintTokensMsg, TokenFactoryMsg};

#[cosmwasm_schema::cw_serde]
#[derive(Enumorph)]
pub enum TokenFactoryAdminOperation {
    MintTokens(MintTokensMsg),
    BurnTokens(BurnTokensMsg),
    ChangeAdmin(ChangeAdminMsg),
}

impl TokenFactoryAdminOperation {
    pub fn into_cosmos_msg(self) -> CosmosMsg<TokenFactoryMsg> {
        CosmosMsg::Custom(match self {
            Self::MintTokens(msg) => TokenFactoryMsg::MintTokens(msg),
            Self::BurnTokens(msg) => TokenFactoryMsg::BurnTokens(msg),
            Self::ChangeAdmin(msg) => TokenFactoryMsg::ChangeAdmin(msg),
        })
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    ChangeTokenOwner {
        denom: String,
        new_owner: Addr,
    },

    #[serde(untagged)]
    ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg),
}
