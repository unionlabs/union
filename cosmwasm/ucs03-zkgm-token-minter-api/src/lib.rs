use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use enumorph::Enumorph;
use token_factory_api::{DenomUnit, TokenFactoryMsg};

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
pub struct Metadata {
    /// denom_units represents the list of DenomUnit's for a given coin
    pub denom_units: Vec<DenomUnit>,
    /// display indicates the suggested denom that should be displayed in clients.
    pub display: String,
    /// name defines the name of the token (eg: Cosmos Atom)
    pub name: String,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    pub symbol: String,
}

impl From<Metadata> for token_factory_api::Metadata {
    fn from(value: Metadata) -> Self {
        Self {
            description: None,
            denom_units: value.denom_units,
            base: None,
            display: Some(value.display),
            name: Some(value.name),
            symbol: Some(value.symbol),
            uri: None,
            uri_hash: None,
        }
    }
}

#[cw_serde]
pub enum WrappedTokenMsg {
    CreateDenom {
        subdenom: String,
        // TODO: upgrade tokenfactory to handle this
        metadata: Metadata,
    },
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
        sender: Addr,
    },
}

impl From<WrappedTokenMsg> for TokenFactoryMsg {
    fn from(value: WrappedTokenMsg) -> Self {
        match value {
            WrappedTokenMsg::CreateDenom { subdenom, metadata } => TokenFactoryMsg::CreateDenom {
                subdenom,
                metadata: Some(metadata.into()),
            },
            WrappedTokenMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            } => TokenFactoryMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            },
            WrappedTokenMsg::BurnTokens {
                denom,
                amount,
                burn_from_address,
                ..
            } => TokenFactoryMsg::BurnTokens {
                denom,
                amount,
                burn_from_address,
            },
        }
    }
}

#[cw_serde]
#[derive(Enumorph)]
pub enum ExecuteMsg {
    Wrapped(WrappedTokenMsg),
    Local(LocalTokenMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Metadata { denom: String },
    BaseToken { base_token: String },
}

#[cw_serde]
pub struct MetadataResponse {
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub struct BaseTokenResponse {
    pub base_token: String,
}
