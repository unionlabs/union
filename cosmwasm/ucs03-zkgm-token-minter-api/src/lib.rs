use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Event, Uint128};
use enumorph::Enumorph;
use ibc_union_spec::ChannelId;
use unionlabs::primitives::{encoding::HexPrefixed, Bytes, H256};

pub const EVENT_WRAPPED_TOKEN: &str = "create_wrapped_token";
pub const EVENT_WRAPPED_TOKEN_ATTR_CHANNEL_ID: &str = "channel_id";
pub const EVENT_WRAPPED_TOKEN_ATTR_PATH: &str = "path";
pub const EVENT_WRAPPED_TOKEN_ATTR_BASE_TOKEN: &str = "base_token";
pub const EVENT_WRAPPED_TOKEN_ATTR_QUOTE_TOKEN: &str = "quote_token";
pub const EVENT_WRAPPED_TOKEN_ATTR_METADATA: &str = "metadata";
pub const EVENT_WRAPPED_TOKEN_ATTR_KIND: &str = "kind";

#[repr(u8)]
pub enum WrappedTokenKind {
    Protocol = 0,
    ThirdParty = 1,
}

pub fn new_wrapped_token_event(
    path: U256,
    channel_id: ChannelId,
    base_token: Vec<u8>,
    quote_token_denom: &str,
    metadata: Vec<u8>,
    kind: WrappedTokenKind,
) -> Event {
    Event::new(EVENT_WRAPPED_TOKEN)
        .add_attribute(EVENT_WRAPPED_TOKEN_ATTR_PATH, path.to_string())
        .add_attribute(EVENT_WRAPPED_TOKEN_ATTR_CHANNEL_ID, channel_id.to_string())
        .add_attribute(
            EVENT_WRAPPED_TOKEN_ATTR_BASE_TOKEN,
            Bytes::<HexPrefixed>::from(base_token).to_string(),
        )
        .add_attribute(EVENT_WRAPPED_TOKEN_ATTR_QUOTE_TOKEN, quote_token_denom)
        .add_attribute(
            EVENT_WRAPPED_TOKEN_ATTR_METADATA,
            Bytes::<HexPrefixed>::from(metadata).to_string(),
        )
        .add_attribute(EVENT_WRAPPED_TOKEN_ATTR_KIND, (kind as u8).to_string())
}

pub fn encode_metadata(implementation: &[u8], initializer: &[u8]) -> Vec<u8> {
    (implementation, initializer).abi_encode_params()
}

#[cw_serde]
pub enum TokenMinterInitMsg {
    /// Cosmwasm's [CW20] minter, will use virtualized `CW20` tokens.
    /// Note that the `CW20` stack is fully on CosmWasm. Similarly to Ethereum's ERC20, the tokens will be
    /// contracts.
    ///
    /// [CW20]: https://github.com/unionlabs/union/blob/2a09e8e6b570292fe117c5009bbf22ca140099ba/cosmwasm/cw20-base/README.md
    Cw20 {
        #[serde(alias = "cw20_base_code_id")]
        cw20_impl_code_id: u64,
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
    /// already has the funds. This means the transfer is always expected to be successful.
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
        subdenom: String,
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
    /// Create a new denom
    ///
    /// - CW20: This should be a predetermined address that can be deterministically calculated. The admin
    /// of the token contract MUST be the given `zkgm_admin`.
    /// - Native: The denom is expected to be the denom of the token. The given decimal is expected to be set in `denom_units`
    /// right after the first `DenomUnit` which always have a `0` exponent.
    CreateDenomV2 {
        /// The full denom that is defined by the underlying minter. Note that this will always
        /// be the output of `PredictWrappedToken` query.
        subdenom: String,
        /// The ZKGM path that is sent by the origin chain
        path: Binary,
        /// The destination(this chain) channel id
        channel_id: ChannelId,
        /// The original token denomination. Different chains can have different denoms, so this is
        /// just arbitrary bytes
        token: Binary,
        /// Custom implementation:
        /// - CW20: abi encoded (admin, code_id)
        /// - Native: abi encoded (denom, metadata)
        implementation: Binary,
        /// Custom initialization message:
        /// - CW20: frissitheto init message
        /// - Native: empty
        initializer: Binary,
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
    PredictWrappedTokenV2 {
        path: String,
        channel_id: ChannelId,
        token: Binary,
        metadata_image: H256,
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
