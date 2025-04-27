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

/// Messages for the funds that are local to this chain
#[cw_serde]
pub enum LocalTokenMsg {
    /// Lock the funds.
    ///
    /// - CW20: the minter should do a transfer to itself. Note that this requires
    /// the proper allowance to be set by the user.
    ///
    /// - Native: the proper funds must be given with the call. ZKGM will pass the funds to the
    /// underlying minter.
    Escrow {
        from: String,
        denom: String,
        recipient: String,
        amount: Uint128,
    },
    /// Unlock the funds.
    ///
    /// - CW20 & Native: the minter should transfer the funds to the user. ZKGM will take care of the user balances,
    /// so the minter should be blindly transferring the funds. Insufficient balance case will be handled by
    /// ZKGM before this call is being made and if there's enough balance, it is guaranteed that the minter
    /// already has the funds. This means the transfer is always expected to be successfull.
    Unescrow {
        denom: String,
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub struct Metadata {
    /// name defines the name of the token (eg: Circle USDC)
    pub name: String,
    /// symbol is the token symbol usually shown on exchanges (eg: USDC)
    pub symbol: String,
    /// note that the decimals field is represented as `exponent` in Cosmos chains
    pub decimals: u8,
}

/// Messages for the funds that are originated in other chains
#[cw_serde]
pub enum WrappedTokenMsg {
    /// Create a new denom
    ///
    /// - CW20: This should be a predetermined address that can be deterministically calculated. The admin
    /// of the token contract MUST be the given `zkgm_admin`.
    /// - Native: The denom is expected to be the denom of the token. The given decimal is expected to be set in `denom_units`
    /// right after the first `DenomUnit` which always have a `0` exponent.
    CreateDenom {
        /// The full denom that is defined by the underlying minter. Note that this will always
        /// be the output of `PredictWrappedToken` query.
        denom: String,
        /// Metadata of the token
        metadata: Metadata,
        /// The ZKGM path that is sent by the origin chain
        path: Binary,
        /// The destination(this chain) channel id
        channel_id: ChannelId,
        /// The original token denomination. Different chains can have different denoms, so this is
        /// just arbitrary bytes
        token: Binary,
    },
    /// Mint tokens
    ///
    /// - CW20 & Native: Nothing fancy, just mint the tokens to the given address. `denom` is defined by the `PredictWrappedToken` query.
    /// ZKGM will handle the user balances.
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: Addr,
    },
    /// Burn tokens
    ///
    /// - CW20 & Native: Burn the tokens from the given address.
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: Addr,
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
