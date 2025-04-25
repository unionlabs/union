use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use enumorph::Enumorph;
use ibc_union_spec::ChannelId;

#[cw_serde]
pub enum TokenMinterInitMsg {
    /// Cosmwasm's [CW20] minter, will use virtualized `CW20` tokens.
    /// Note that the `CW20` stack is fully on CosmWasm. Similarly to Ethereum's ERC20, the tokens will be
    /// contracts.
    ///
    /// [CW20]: https://github.com/unionlabs/union/blob/2a09e8e6b570292fe117c5009bbf22ca140099ba/cosmwasm/cw20-base/README.md
    Cw20 {
        cw20_base_code_id: u64,
        dummy_code_id: u64,
        zkgm_admin: Addr,
    },
    /// [Osmosis Token Factory] minter, will use Osmosis' Token Factory module to manage tokens
    /// Note that, this will result in the wrapped tokens to be created as native tokens.
    ///
    /// [Osmosis Token Factory]: https://github.com/osmosis-labs/osmosis/blob/e14ace31b7ba46be3d519966fb8563127534b245/x/tokenfactory/README.md
    OsmosisTokenFactory { zkgm_admin: Addr },
}

#[cw_serde]
pub enum LocalTokenMsg {
    Escrow {
        from: String,
        denom: String,
        recipient: String,
        amount: Uint128,
    },
    Unescrow {
        denom: String,
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub struct Metadata {
    /// name defines the name of the token (eg: Cosmos Atom)
    pub name: String,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    pub symbol: String,
    pub decimals: u8,
}

#[cw_serde]
pub enum WrappedTokenMsg {
    CreateDenom {
        subdenom: String,
        // TODO: upgrade tokenfactory to handle this
        metadata: Metadata,
        path: Binary,
        channel_id: ChannelId,
        token: Binary,
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

#[cw_serde]
#[derive(Enumorph)]
pub enum ExecuteMsg {
    Wrapped(WrappedTokenMsg),
    Local(LocalTokenMsg),
}

#[cw_serde]
pub enum QueryMsg {
    /// Query the metadata of a token.
    Metadata {
        /// `denom` is either a normal token denom, or a cosmwasm contract address of a cw20 token that was created through the `cw20-token-minter`.
        denom: String,
    },
    PredictWrappedToken {
        path: String,
        channel_id: ChannelId,
        token: Binary,
    },
}

#[cw_serde]
pub struct MetadataResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[cw_serde]
pub struct PredictWrappedTokenResponse {
    pub wrapped_token: String,
}
