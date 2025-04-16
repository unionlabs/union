// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapAmountInRoute {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_out_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SwapAmountInRoute {
    const NAME: &'static str = "SwapAmountInRoute";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapAmountOutRoute {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SwapAmountOutRoute {
    const NAME: &'static str = "SwapAmountOutRoute";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapAmountInSplitRoute {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
    #[prost(string, tag = "2")]
    pub token_in_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for SwapAmountInSplitRoute {
    const NAME: &'static str = "SwapAmountInSplitRoute";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapAmountOutSplitRoute {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "2")]
    pub token_out_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for SwapAmountOutSplitRoute {
    const NAME: &'static str = "SwapAmountOutSplitRoute";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ModuleRouter defines a route encapsulating pool type.
/// It is used as the value of a mapping from pool id to the pool type,
/// allowing the pool manager to know which module to route swaps to given the
/// pool id.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleRoute {
    /// pool_type specifies the type of the pool
    #[prost(enumeration = "PoolType", tag = "1")]
    pub pool_type: i32,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
}
impl ::prost::Name for ModuleRoute {
    const NAME: &'static str = "ModuleRoute";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// PoolType is an enumeration of all supported pool types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PoolType {
    /// Balancer is the standard xy=k curve. Its pool model is defined in x/gamm.
    Balancer = 0,
    /// Stableswap is the Solidly cfmm stable swap curve. Its pool model is defined
    /// in x/gamm.
    Stableswap = 1,
    /// Concentrated is the pool model specific to concentrated liquidity. It is
    /// defined in x/concentrated-liquidity.
    Concentrated = 2,
    /// CosmWasm is the pool model specific to CosmWasm. It is defined in
    /// x/cosmwasmpool.
    CosmWasm = 3,
}
impl PoolType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PoolType::Balancer => "Balancer",
            PoolType::Stableswap => "Stableswap",
            PoolType::Concentrated => "Concentrated",
            PoolType::CosmWasm => "CosmWasm",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Balancer" => Some(Self::Balancer),
            "Stableswap" => Some(Self::Stableswap),
            "Concentrated" => Some(Self::Concentrated),
            "CosmWasm" => Some(Self::CosmWasm),
            _ => None,
        }
    }
}
/// ===================== MsgSwapExactAmountIn
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
    #[prost(message, optional, tag = "3")]
    pub token_in: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "4")]
    pub token_out_min_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSwapExactAmountIn {
    const NAME: &'static str = "MsgSwapExactAmountIn";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSwapExactAmountInResponse {
    const NAME: &'static str = "MsgSwapExactAmountInResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSplitRouteSwapExactAmountIn
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSplitRouteSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInSplitRoute>,
    #[prost(string, tag = "3")]
    pub token_in_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub token_out_min_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSplitRouteSwapExactAmountIn {
    const NAME: &'static str = "MsgSplitRouteSwapExactAmountIn";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSplitRouteSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSplitRouteSwapExactAmountInResponse {
    const NAME: &'static str = "MsgSplitRouteSwapExactAmountInResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSwapExactAmountOut
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountOut {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "3")]
    pub token_in_max_amount: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub token_out: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgSwapExactAmountOut {
    const NAME: &'static str = "MsgSwapExactAmountOut";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSwapExactAmountOutResponse {
    const NAME: &'static str = "MsgSwapExactAmountOutResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSplitRouteSwapExactAmountOut
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSplitRouteSwapExactAmountOut {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutSplitRoute>,
    #[prost(string, tag = "3")]
    pub token_out_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub token_in_max_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSplitRouteSwapExactAmountOut {
    const NAME: &'static str = "MsgSplitRouteSwapExactAmountOut";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSplitRouteSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSplitRouteSwapExactAmountOutResponse {
    const NAME: &'static str = "MsgSplitRouteSwapExactAmountOutResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSetDenomPairTakerFee
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDenomPairTakerFee {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub denom_pair_taker_fee: ::prost::alloc::vec::Vec<DenomPairTakerFee>,
}
impl ::prost::Name for MsgSetDenomPairTakerFee {
    const NAME: &'static str = "MsgSetDenomPairTakerFee";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDenomPairTakerFeeResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgSetDenomPairTakerFeeResponse {
    const NAME: &'static str = "MsgSetDenomPairTakerFeeResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSetTakerFeeShareAgreementForDenom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetTakerFeeShareAgreementForDenom {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// denom is the denom that the taker fee share agreement is being set for.
    /// Ex. If this is set to "nBTC", then any trade route that includes "nBTC"
    /// will have the skim_percent skimmed from the taker fees and sent to the
    /// skim_address.
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    /// skim_percent is the percentage of taker fees that will be skimmed for the
    /// bridge provider, in the event that the bridge provider's denom is included
    /// in the swap route.
    #[prost(string, tag = "3")]
    pub skim_percent: ::prost::alloc::string::String,
    /// skim_address is the address belonging to the respective bridge provider
    /// that the skimmed taker fees will be sent to at the end of each epoch.
    #[prost(string, tag = "4")]
    pub skim_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetTakerFeeShareAgreementForDenom {
    const NAME: &'static str = "MsgSetTakerFeeShareAgreementForDenom";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetTakerFeeShareAgreementForDenomResponse {}
impl ::prost::Name for MsgSetTakerFeeShareAgreementForDenomResponse {
    const NAME: &'static str = "MsgSetTakerFeeShareAgreementForDenomResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSetRegisteredAlloyedPool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetRegisteredAlloyedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// pool_id is the id of the pool that is being registered as an alloyed pool.
    /// Only alloyed pools that intend to be used in taker fee revenue sharing
    /// should be registered.
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgSetRegisteredAlloyedPool {
    const NAME: &'static str = "MsgSetRegisteredAlloyedPool";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetRegisteredAlloyedPoolResponse {}
impl ::prost::Name for MsgSetRegisteredAlloyedPoolResponse {
    const NAME: &'static str = "MsgSetRegisteredAlloyedPoolResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomPairTakerFee {
    /// DEPRECATED: Now that we are using uni-directional trading pairs, we are
    /// using tokenInDenom and tokenOutDenom instead of denom0 and denom1 to
    /// prevent confusion.
    #[deprecated]
    #[prost(string, tag = "1")]
    pub denom0: ::prost::alloc::string::String,
    #[deprecated]
    #[prost(string, tag = "2")]
    pub denom1: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub taker_fee: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub token_in_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub token_out_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for DenomPairTakerFee {
    const NAME: &'static str = "DenomPairTakerFee";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// Params holds parameters for the poolmanager module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(message, repeated, tag = "1")]
    pub pool_creation_fee:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// taker_fee_params is the container of taker fee parameters.
    #[prost(message, optional, tag = "2")]
    pub taker_fee_params: ::core::option::Option<TakerFeeParams>,
    /// authorized_quote_denoms is a list of quote denoms that can be used as
    /// token1 when creating a concentrated pool. We limit the quote assets to a
    /// small set for the purposes of having convenient price increments stemming
    /// from tick to price conversion. These increments are in a human readable
    /// magnitude only for token1 as a quote. For limit orders in the future, this
    /// will be a desirable property in terms of UX as to allow users to set limit
    /// orders at prices in terms of token1 (quote asset) that are easy to reason
    /// about.
    #[prost(string, repeated, tag = "3")]
    pub authorized_quote_denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the poolmanager module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// the next_pool_id
    #[prost(uint64, tag = "1")]
    pub next_pool_id: u64,
    /// params is the container of poolmanager parameters.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
    /// pool_routes is the container of the mappings from pool id to pool type.
    #[prost(message, repeated, tag = "3")]
    pub pool_routes: ::prost::alloc::vec::Vec<ModuleRoute>,
    /// KVStore state
    #[prost(message, optional, tag = "4")]
    pub taker_fees_tracker: ::core::option::Option<TakerFeesTracker>,
    #[prost(message, repeated, tag = "5")]
    pub pool_volumes: ::prost::alloc::vec::Vec<PoolVolume>,
    #[prost(message, repeated, tag = "6")]
    pub denom_pair_taker_fee_store: ::prost::alloc::vec::Vec<DenomPairTakerFee>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// TakerFeeParams consolidates the taker fee parameters for the poolmanager.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeParams {
    /// default_taker_fee is the fee used when creating a new pool that doesn't
    /// fall under a custom pool taker fee or stableswap taker fee category.
    #[prost(string, tag = "1")]
    pub default_taker_fee: ::prost::alloc::string::String,
    /// osmo_taker_fee_distribution defines the distribution of taker fees
    /// generated in OSMO. As of this writing, it has two categories:
    /// - staking_rewards: the percent of the taker fee that gets distributed to
    ///    stakers.
    /// - community_pool: the percent of the taker fee that gets sent to the
    ///    community pool.
    #[prost(message, optional, tag = "2")]
    pub osmo_taker_fee_distribution: ::core::option::Option<TakerFeeDistributionPercentage>,
    /// non_osmo_taker_fee_distribution defines the distribution of taker fees
    /// generated in non-OSMO. As of this writing, it has two categories:
    /// - staking_rewards: the percent of the taker fee that gets swapped to OSMO
    ///    and then distributed to stakers.
    /// - community_pool: the percent of the taker fee that gets sent to the
    ///    community pool. Note: If the non-OSMO asset is an authorized_quote_denom,
    ///    that denom is sent directly to the community pool. Otherwise, it is
    ///    swapped to the community_pool_denom_to_swap_non_whitelisted_assets_to and
    ///    then sent to the community pool as that denom.
    #[prost(message, optional, tag = "3")]
    pub non_osmo_taker_fee_distribution: ::core::option::Option<TakerFeeDistributionPercentage>,
    /// admin_addresses is a list of addresses that are allowed to set and remove
    /// custom taker fees for denom pairs. Governance also has the ability to set
    /// and remove custom taker fees for denom pairs, but with the normal
    /// governance delay.
    #[prost(string, repeated, tag = "4")]
    pub admin_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// community_pool_denom_to_swap_non_whitelisted_assets_to is the denom that
    /// non-whitelisted taker fees will be swapped to before being sent to
    /// the community pool.
    #[prost(string, tag = "5")]
    pub community_pool_denom_to_swap_non_whitelisted_assets_to: ::prost::alloc::string::String,
    /// reduced_fee_whitelist is a list of addresses that are
    /// allowed to pay a reduce taker fee when performing a swap
    /// (i.e. swap without paying the taker fee).
    /// It is intended to be used for integrators who meet qualifying factors
    /// that are approved by governance.
    /// Initially, the taker fee is allowed to be bypassed completely. However
    /// In the future, we will charge a reduced taker fee instead of no fee at all.
    #[prost(string, repeated, tag = "6")]
    pub reduced_fee_whitelist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for TakerFeeParams {
    const NAME: &'static str = "TakerFeeParams";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// TakerFeeDistributionPercentage defines what percent of the taker fee category
/// gets distributed to the available categories.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeDistributionPercentage {
    #[prost(string, tag = "1")]
    pub staking_rewards: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub community_pool: ::prost::alloc::string::String,
}
impl ::prost::Name for TakerFeeDistributionPercentage {
    const NAME: &'static str = "TakerFeeDistributionPercentage";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeesTracker {
    #[prost(message, repeated, tag = "1")]
    pub taker_fees_to_stakers:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "2")]
    pub taker_fees_to_community_pool:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(int64, tag = "3")]
    pub height_accounting_starts_from: i64,
}
impl ::prost::Name for TakerFeesTracker {
    const NAME: &'static str = "TakerFeesTracker";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// PoolVolume stores the KVStore entries for each pool's volume, which
/// is used in export/import genesis.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolVolume {
    /// pool_id is the id of the pool.
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    /// pool_volume is the cumulative volume of the pool.
    #[prost(message, repeated, tag = "2")]
    pub pool_volume: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for PoolVolume {
    const NAME: &'static str = "PoolVolume";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// DenomPairTakerFeeProposal is a type for adding/removing a custom taker fee(s)
/// for one or more denom pairs.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomPairTakerFeeProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub denom_pair_taker_fee: ::prost::alloc::vec::Vec<DenomPairTakerFee>,
}
impl ::prost::Name for DenomPairTakerFeeProposal {
    const NAME: &'static str = "DenomPairTakerFeeProposal";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// TakerFeeShareAgreement represents the agreement between the Osmosis protocol
/// and a specific denom to share a certain percent of taker fees generated in
/// any route that contains said denom. For example, if the agreement specifies a
/// 10% skim_percent, this means 10% of the taker fees generated in a swap route
/// containing the specified denom will be sent to the address specified
/// in the skim_address field at the end of each epoch. These skim_percents are
/// additive, so if three taker fee agreements have skim percents of 10%, 20%,
/// and 30%, the total skim percent for the route will be 60%.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeShareAgreement {
    /// denom is the denom that has the taker fee share agreement.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// skim_percent is the percentage of taker fees that will be skimmed for the
    /// denom, in the event that the denom is included in the swap route.
    #[prost(string, tag = "2")]
    pub skim_percent: ::prost::alloc::string::String,
    /// skim_address is the address belonging to the respective denom
    /// that the skimmed taker fees will be sent to at the end of each epoch.
    #[prost(string, tag = "3")]
    pub skim_address: ::prost::alloc::string::String,
}
impl ::prost::Name for TakerFeeShareAgreement {
    const NAME: &'static str = "TakerFeeShareAgreement";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// TakerFeeSkimAccumulator accumulates the total skimmed taker fees for each
/// denom that has a taker fee share agreement.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeSkimAccumulator {
    /// denom is the denom that has the taker fee share agreement.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// skimmed_taker_fees is the total skimmed taker fees for the denom.
    #[prost(message, repeated, tag = "2")]
    pub skimmed_taker_fees:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TakerFeeSkimAccumulator {
    const NAME: &'static str = "TakerFeeSkimAccumulator";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// AlloyContractTakerFeeShareState contains the contract address of the alloyed
/// asset pool, along with the adjusted taker fee share agreements for any asset
/// within the alloyed asset pool that has a taker fee share agreement. If for
/// instance there are two denoms, and denomA makes up 50 percent and denomB
/// makes up 50 percent, and denom A has a taker fee share agreement with a skim
/// percent of 10%, then the adjusted taker fee share agreement for denomA will
/// be 5%.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AlloyContractTakerFeeShareState {
    /// contract_address is the address of the alloyed asset pool contract.
    #[prost(string, tag = "1")]
    pub contract_address: ::prost::alloc::string::String,
    /// taker_fee_share_agreements is the adjusted taker fee share agreements for
    /// any asset within the alloyed asset pool that has a taker fee share
    /// agreement.
    #[prost(message, repeated, tag = "2")]
    pub taker_fee_share_agreements: ::prost::alloc::vec::Vec<TakerFeeShareAgreement>,
}
impl ::prost::Name for AlloyContractTakerFeeShareState {
    const NAME: &'static str = "AlloyContractTakerFeeShareState";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== Params
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsRequest {}
impl ::prost::Name for ParamsRequest {
    const NAME: &'static str = "ParamsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for ParamsResponse {
    const NAME: &'static str = "ParamsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== EstimateSwapExactAmountIn
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountInRequest {
    /// DEPRECATED
    #[deprecated]
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[deprecated]
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(string, tag = "3")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
}
impl ::prost::Name for EstimateSwapExactAmountInRequest {
    const NAME: &'static str = "EstimateSwapExactAmountInRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountInWithPrimitiveTypesRequest {
    #[deprecated]
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub routes_pool_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, repeated, tag = "4")]
    pub routes_token_out_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for EstimateSwapExactAmountInWithPrimitiveTypesRequest {
    const NAME: &'static str = "EstimateSwapExactAmountInWithPrimitiveTypesRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSinglePoolSwapExactAmountInRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub token_out_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSinglePoolSwapExactAmountInRequest {
    const NAME: &'static str = "EstimateSinglePoolSwapExactAmountInRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSwapExactAmountInResponse {
    const NAME: &'static str = "EstimateSwapExactAmountInResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== EstimateSwapExactAmountOut
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountOutRequest {
    /// DEPRECATED
    #[deprecated]
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[deprecated]
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(message, repeated, tag = "3")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "4")]
    pub token_out: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSwapExactAmountOutRequest {
    const NAME: &'static str = "EstimateSwapExactAmountOutRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountOutWithPrimitiveTypesRequest {
    #[deprecated]
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(uint64, repeated, packed = "false", tag = "2")]
    pub routes_pool_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, repeated, tag = "3")]
    pub routes_token_in_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "4")]
    pub token_out: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSwapExactAmountOutWithPrimitiveTypesRequest {
    const NAME: &'static str = "EstimateSwapExactAmountOutWithPrimitiveTypesRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSinglePoolSwapExactAmountOutRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSinglePoolSwapExactAmountOutRequest {
    const NAME: &'static str = "EstimateSinglePoolSwapExactAmountOutRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSwapExactAmountOutResponse {
    const NAME: &'static str = "EstimateSwapExactAmountOutResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== NumPools
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NumPoolsRequest {}
impl ::prost::Name for NumPoolsRequest {
    const NAME: &'static str = "NumPoolsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NumPoolsResponse {
    #[prost(uint64, tag = "1")]
    pub num_pools: u64,
}
impl ::prost::Name for NumPoolsResponse {
    const NAME: &'static str = "NumPoolsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== Pool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for PoolRequest {
    const NAME: &'static str = "PoolRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolResponse {
    #[prost(message, optional, tag = "1")]
    pub pool: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for PoolResponse {
    const NAME: &'static str = "PoolResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== AllPools
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllPoolsRequest {}
impl ::prost::Name for AllPoolsRequest {
    const NAME: &'static str = "AllPoolsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllPoolsResponse {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for AllPoolsResponse {
    const NAME: &'static str = "AllPoolsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =======================================================
/// ListPoolsByDenomRequest
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPoolsByDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for ListPoolsByDenomRequest {
    const NAME: &'static str = "ListPoolsByDenomRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPoolsByDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for ListPoolsByDenomResponse {
    const NAME: &'static str = "ListPoolsByDenomResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// ==========================================================
/// SpotPriceRequest defines the gRPC request structure for a SpotPrice
/// query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotPriceRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SpotPriceRequest {
    const NAME: &'static str = "SpotPriceRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// SpotPriceResponse defines the gRPC response structure for a SpotPrice
/// query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotPriceResponse {
    /// String of the Dec. Ex) 10.203uatom
    #[prost(string, tag = "1")]
    pub spot_price: ::prost::alloc::string::String,
}
impl ::prost::Name for SpotPriceResponse {
    const NAME: &'static str = "SpotPriceResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== TotalPoolLiquidity
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalPoolLiquidityRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for TotalPoolLiquidityRequest {
    const NAME: &'static str = "TotalPoolLiquidityRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalPoolLiquidityResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TotalPoolLiquidityResponse {
    const NAME: &'static str = "TotalPoolLiquidityResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== TotalLiquidity
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalLiquidityRequest {}
impl ::prost::Name for TotalLiquidityRequest {
    const NAME: &'static str = "TotalLiquidityRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalLiquidityResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TotalLiquidityResponse {
    const NAME: &'static str = "TotalLiquidityResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== TotalVolumeForPool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalVolumeForPoolRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for TotalVolumeForPoolRequest {
    const NAME: &'static str = "TotalVolumeForPoolRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalVolumeForPoolResponse {
    #[prost(message, repeated, tag = "1")]
    pub volume: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TotalVolumeForPoolResponse {
    const NAME: &'static str = "TotalVolumeForPoolResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// =============================== TradingPairTakerFee
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingPairTakerFeeRequest {
    #[prost(string, tag = "1")]
    pub denom_0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom_1: ::prost::alloc::string::String,
}
impl ::prost::Name for TradingPairTakerFeeRequest {
    const NAME: &'static str = "TradingPairTakerFeeRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingPairTakerFeeResponse {
    #[prost(string, tag = "1")]
    pub taker_fee: ::prost::alloc::string::String,
}
impl ::prost::Name for TradingPairTakerFeeResponse {
    const NAME: &'static str = "TradingPairTakerFeeResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// EstimateTradeBasedOnPriceImpactRequest represents a request to estimate a
/// trade for Balancer/StableSwap/Concentrated liquidity pool types based on the
/// given parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateTradeBasedOnPriceImpactRequest {
    /// from_coin is the total amount of tokens that the user wants to sell.
    #[prost(message, optional, tag = "1")]
    pub from_coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    /// to_coin_denom is the denom identifier of the token that the user wants to
    /// buy.
    #[prost(string, tag = "2")]
    pub to_coin_denom: ::prost::alloc::string::String,
    /// pool_id is the identifier of the liquidity pool that the trade will occur
    /// on.
    #[prost(uint64, tag = "3")]
    pub pool_id: u64,
    /// max_price_impact is the maximum percentage that the user is willing
    /// to affect the price of the liquidity pool.
    #[prost(string, tag = "4")]
    pub max_price_impact: ::prost::alloc::string::String,
    /// external_price is an optional external price that the user can enter.
    /// It adjusts the MaxPriceImpact as the SpotPrice of a pool can be changed at
    /// any time.
    #[prost(string, tag = "5")]
    pub external_price: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateTradeBasedOnPriceImpactRequest {
    const NAME: &'static str = "EstimateTradeBasedOnPriceImpactRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
/// EstimateTradeBasedOnPriceImpactResponse represents the response data
/// for an estimated trade based on price impact. If a trade fails to be
/// estimated the response would be 0,0 for input_coin and output_coin and will
/// not error.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateTradeBasedOnPriceImpactResponse {
    /// input_coin is the actual input amount that would be tradeable
    /// under the specified price impact.
    #[prost(message, optional, tag = "1")]
    pub input_coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    /// output_coin is the amount of tokens of the ToCoinDenom type
    /// that will be received for the actual InputCoin trade.
    #[prost(message, optional, tag = "2")]
    pub output_coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for EstimateTradeBasedOnPriceImpactResponse {
    const NAME: &'static str = "EstimateTradeBasedOnPriceImpactResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTakerFeeShareAgreementsRequest {}
impl ::prost::Name for AllTakerFeeShareAgreementsRequest {
    const NAME: &'static str = "AllTakerFeeShareAgreementsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTakerFeeShareAgreementsResponse {
    #[prost(message, repeated, tag = "1")]
    pub taker_fee_share_agreements: ::prost::alloc::vec::Vec<TakerFeeShareAgreement>,
}
impl ::prost::Name for AllTakerFeeShareAgreementsResponse {
    const NAME: &'static str = "AllTakerFeeShareAgreementsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeShareAgreementFromDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for TakerFeeShareAgreementFromDenomRequest {
    const NAME: &'static str = "TakerFeeShareAgreementFromDenomRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeShareAgreementFromDenomResponse {
    #[prost(message, optional, tag = "1")]
    pub taker_fee_share_agreement: ::core::option::Option<TakerFeeShareAgreement>,
}
impl ::prost::Name for TakerFeeShareAgreementFromDenomResponse {
    const NAME: &'static str = "TakerFeeShareAgreementFromDenomResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeShareDenomsToAccruedValueRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub taker_fee_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for TakerFeeShareDenomsToAccruedValueRequest {
    const NAME: &'static str = "TakerFeeShareDenomsToAccruedValueRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TakerFeeShareDenomsToAccruedValueResponse {
    #[prost(string, tag = "1")]
    pub amount: ::prost::alloc::string::String,
}
impl ::prost::Name for TakerFeeShareDenomsToAccruedValueResponse {
    const NAME: &'static str = "TakerFeeShareDenomsToAccruedValueResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTakerFeeShareAccumulatorsRequest {}
impl ::prost::Name for AllTakerFeeShareAccumulatorsRequest {
    const NAME: &'static str = "AllTakerFeeShareAccumulatorsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTakerFeeShareAccumulatorsResponse {
    #[prost(message, repeated, tag = "1")]
    pub taker_fee_skim_accumulators: ::prost::alloc::vec::Vec<TakerFeeSkimAccumulator>,
}
impl ::prost::Name for AllTakerFeeShareAccumulatorsResponse {
    const NAME: &'static str = "AllTakerFeeShareAccumulatorsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredAlloyedPoolFromDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for RegisteredAlloyedPoolFromDenomRequest {
    const NAME: &'static str = "RegisteredAlloyedPoolFromDenomRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredAlloyedPoolFromDenomResponse {
    #[prost(message, optional, tag = "1")]
    pub contract_state: ::core::option::Option<AlloyContractTakerFeeShareState>,
}
impl ::prost::Name for RegisteredAlloyedPoolFromDenomResponse {
    const NAME: &'static str = "RegisteredAlloyedPoolFromDenomResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredAlloyedPoolFromPoolIdRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for RegisteredAlloyedPoolFromPoolIdRequest {
    const NAME: &'static str = "RegisteredAlloyedPoolFromPoolIdRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredAlloyedPoolFromPoolIdResponse {
    #[prost(message, optional, tag = "1")]
    pub contract_state: ::core::option::Option<AlloyContractTakerFeeShareState>,
}
impl ::prost::Name for RegisteredAlloyedPoolFromPoolIdResponse {
    const NAME: &'static str = "RegisteredAlloyedPoolFromPoolIdResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllRegisteredAlloyedPoolsRequest {}
impl ::prost::Name for AllRegisteredAlloyedPoolsRequest {
    const NAME: &'static str = "AllRegisteredAlloyedPoolsRequest";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllRegisteredAlloyedPoolsResponse {
    #[prost(message, repeated, tag = "1")]
    pub contract_states: ::prost::alloc::vec::Vec<AlloyContractTakerFeeShareState>,
}
impl ::prost::Name for AllRegisteredAlloyedPoolsResponse {
    const NAME: &'static str = "AllRegisteredAlloyedPoolsResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TrackedVolume {
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TrackedVolume {
    const NAME: &'static str = "TrackedVolume";
    const PACKAGE: &'static str = "osmosis.poolmanager.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.poolmanager.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
