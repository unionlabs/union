// @generated
/// TokenPairArbRoutes tracks all of the hot routes for a given pair of tokens
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TokenPairArbRoutes {
    /// Stores all of the possible hot paths for a given pair of tokens
    #[prost(message, repeated, tag = "1")]
    pub arb_routes: ::prost::alloc::vec::Vec<Route>,
    /// Token denomination of the first asset
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    /// Token denomination of the second asset
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
impl ::prost::Name for TokenPairArbRoutes {
    const NAME: &'static str = "TokenPairArbRoutes";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// Route is a hot route for a given pair of tokens
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Route {
    /// The pool IDs that are traversed in the directed cyclic graph (traversed
    /// left
    /// -> right)
    #[prost(message, repeated, tag = "1")]
    pub trades: ::prost::alloc::vec::Vec<Trade>,
    /// The step size that will be used to find the optimal swap amount in the
    /// binary search
    #[prost(string, tag = "2")]
    pub step_size: ::prost::alloc::string::String,
}
impl ::prost::Name for Route {
    const NAME: &'static str = "Route";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// Trade is a single trade in a route
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trade {
    /// The pool id of the pool that is traded on
    #[prost(uint64, tag = "1")]
    pub pool: u64,
    /// The denom of the token that is traded
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    /// The denom of the token that is received
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
impl ::prost::Name for Trade {
    const NAME: &'static str = "Trade";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// RouteStatistics contains the number of trades the module has executed after a
/// swap on a given route and the profits from the trades
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RouteStatistics {
    /// profits is the total profit from all trades on this route
    #[prost(message, repeated, tag = "1")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// number_of_trades is the number of trades the module has executed using this
    /// route
    #[prost(string, tag = "2")]
    pub number_of_trades: ::prost::alloc::string::String,
    /// route is the route that was used (pool ids along the arbitrage route)
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub route: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for RouteStatistics {
    const NAME: &'static str = "RouteStatistics";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// PoolWeights contains the weights of all of the different pool types. This
/// distinction is made and necessary because the execution time ranges
/// significantly between the different pool types. Each weight roughly
/// corresponds to the amount of time (in ms) it takes to execute a swap on that
/// pool type.
///
/// DEPRECATED: This field is deprecated and will be removed in the next
/// release. It is replaced by the `info_by_pool_type` field.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolWeights {
    /// The weight of a stableswap pool
    #[prost(uint64, tag = "1")]
    pub stable_weight: u64,
    /// The weight of a balancer pool
    #[prost(uint64, tag = "2")]
    pub balancer_weight: u64,
    /// The weight of a concentrated pool
    #[prost(uint64, tag = "3")]
    pub concentrated_weight: u64,
    /// The weight of a cosmwasm pool
    #[prost(uint64, tag = "4")]
    pub cosmwasm_weight: u64,
}
impl ::prost::Name for PoolWeights {
    const NAME: &'static str = "PoolWeights";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// InfoByPoolType contains information pertaining to how expensive (in terms of
/// gas and time) it is to execute a swap on a given pool type. This distinction
/// is made and necessary because the execution time ranges significantly between
/// the different pool types.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InfoByPoolType {
    /// The stable pool info
    #[prost(message, optional, tag = "1")]
    pub stable: ::core::option::Option<StablePoolInfo>,
    /// The balancer pool info
    #[prost(message, optional, tag = "2")]
    pub balancer: ::core::option::Option<BalancerPoolInfo>,
    /// The concentrated pool info
    #[prost(message, optional, tag = "3")]
    pub concentrated: ::core::option::Option<ConcentratedPoolInfo>,
    /// The cosmwasm pool info
    #[prost(message, optional, tag = "4")]
    pub cosmwasm: ::core::option::Option<CosmwasmPoolInfo>,
}
impl ::prost::Name for InfoByPoolType {
    const NAME: &'static str = "InfoByPoolType";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// StablePoolInfo contains meta data pertaining to a stableswap pool type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StablePoolInfo {
    /// The weight of a stableswap pool
    #[prost(uint64, tag = "1")]
    pub weight: u64,
}
impl ::prost::Name for StablePoolInfo {
    const NAME: &'static str = "StablePoolInfo";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// BalancerPoolInfo contains meta data pertaining to a balancer pool type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalancerPoolInfo {
    /// The weight of a balancer pool
    #[prost(uint64, tag = "1")]
    pub weight: u64,
}
impl ::prost::Name for BalancerPoolInfo {
    const NAME: &'static str = "BalancerPoolInfo";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// ConcentratedPoolInfo contains meta data pertaining to a concentrated pool
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConcentratedPoolInfo {
    /// The weight of a concentrated pool
    #[prost(uint64, tag = "1")]
    pub weight: u64,
    /// The maximum number of ticks we can move when rebalancing
    #[prost(uint64, tag = "2")]
    pub max_ticks_crossed: u64,
}
impl ::prost::Name for ConcentratedPoolInfo {
    const NAME: &'static str = "ConcentratedPoolInfo";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// CosmwasmPoolInfo contains meta data pertaining to a cosmwasm pool type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CosmwasmPoolInfo {
    /// The weight of a cosmwasm pool (by contract address)
    #[prost(message, repeated, tag = "1")]
    pub weight_maps: ::prost::alloc::vec::Vec<WeightMap>,
}
impl ::prost::Name for CosmwasmPoolInfo {
    const NAME: &'static str = "CosmwasmPoolInfo";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// WeightMap maps a contract address to a weight. The weight of an address
/// corresponds to the amount of ms required to execute a swap on that contract.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WeightMap {
    /// The weight of a cosmwasm pool (by contract address)
    #[prost(uint64, tag = "1")]
    pub weight: u64,
    /// The contract address
    #[prost(string, tag = "2")]
    pub contract_address: ::prost::alloc::string::String,
}
impl ::prost::Name for WeightMap {
    const NAME: &'static str = "WeightMap";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// BaseDenom represents a single base denom that the module uses for its
/// arbitrage trades. It contains the denom name alongside the step size of the
/// binary search that is used to find the optimal swap amount
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseDenom {
    /// The denom i.e. name of the base denom (ex. uosmo)
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// The step size of the binary search that is used to find the optimal swap
    /// amount
    #[prost(string, tag = "2")]
    pub step_size: ::prost::alloc::string::String,
}
impl ::prost::Name for BaseDenom {
    const NAME: &'static str = "BaseDenom";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// BaseDenoms represents all of the base denoms that the module uses for its
/// arbitrage trades.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseDenoms {
    #[prost(message, repeated, tag = "1")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
}
impl ::prost::Name for BaseDenoms {
    const NAME: &'static str = "BaseDenoms";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllProtocolRevenue {
    #[prost(message, optional, tag = "1")]
    pub taker_fees_tracker:
        ::core::option::Option<super::super::poolmanager::v1beta1::TakerFeesTracker>,
    /// DEPRECATED
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub tx_fees_tracker: ::core::option::Option<super::super::txfees::v1beta1::TxFeesTracker>,
    #[prost(message, optional, tag = "3")]
    pub cyclic_arb_tracker: ::core::option::Option<CyclicArbTracker>,
}
impl ::prost::Name for AllProtocolRevenue {
    const NAME: &'static str = "AllProtocolRevenue";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CyclicArbTracker {
    #[prost(message, repeated, tag = "1")]
    pub cyclic_arb: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(int64, tag = "2")]
    pub height_accounting_starts_from: i64,
}
impl ::prost::Name for CyclicArbTracker {
    const NAME: &'static str = "CyclicArbTracker";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// Boolean whether the protorev module is enabled.
    #[prost(bool, tag = "1")]
    pub enabled: bool,
    /// The admin account (settings manager) of the protorev module.
    #[prost(string, tag = "2")]
    pub admin: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the protorev module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// Parameters for the protorev module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// Token pair arb routes for the protorev module (hot routes).
    #[prost(message, repeated, tag = "2")]
    pub token_pair_arb_routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
    /// The base denominations being used to create cyclic arbitrage routes via the
    /// highest liquidity method.
    #[prost(message, repeated, tag = "3")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
    /// DEPRECATED: pool_weights are weights that are being used to calculate the
    /// compute cost of each route. This field is deprecated.
    /// It is replaced by the `info_by_pool_type` field.
    #[deprecated]
    #[prost(message, optional, tag = "4")]
    pub pool_weights: ::core::option::Option<PoolWeights>,
    /// The number of days since module genesis.
    #[prost(uint64, tag = "5")]
    pub days_since_module_genesis: u64,
    /// The fees the developer account has accumulated over time.
    #[prost(message, repeated, tag = "6")]
    pub developer_fees: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// The latest block height that the module has processed.
    #[prost(uint64, tag = "7")]
    pub latest_block_height: u64,
    /// The developer account address of the module.
    #[prost(string, tag = "8")]
    pub developer_address: ::prost::alloc::string::String,
    /// Max pool points per block i.e. the maximum compute time (in ms)
    /// that protorev can use per block.
    #[prost(uint64, tag = "9")]
    pub max_pool_points_per_block: u64,
    /// Max pool points per tx i.e. the maximum compute time (in ms) that
    /// protorev can use per tx.
    #[prost(uint64, tag = "10")]
    pub max_pool_points_per_tx: u64,
    /// The number of pool points that have been consumed in the current block.
    #[prost(uint64, tag = "11")]
    pub point_count_for_block: u64,
    /// All of the profits that have been accumulated by the module.
    #[prost(message, repeated, tag = "12")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Information that is used to estimate execution time / gas
    /// consumption of a swap on a given pool type.
    #[prost(message, optional, tag = "13")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
    #[prost(message, optional, tag = "14")]
    pub cyclic_arb_tracker: ::core::option::Option<CyclicArbTracker>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// SetProtoRevEnabledProposal is a gov Content type to update whether the
/// protorev module is enabled
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetProtoRevEnabledProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub enabled: bool,
}
impl ::prost::Name for SetProtoRevEnabledProposal {
    const NAME: &'static str = "SetProtoRevEnabledProposal";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// SetProtoRevAdminAccountProposal is a gov Content type to set the admin
/// account that will receive permissions to alter hot routes and set the
/// developer address that will be receiving a share of profits from the module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetProtoRevAdminAccountProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account: ::prost::alloc::string::String,
}
impl ::prost::Name for SetProtoRevAdminAccountProposal {
    const NAME: &'static str = "SetProtoRevAdminAccountProposal";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevNumberOfTradesRequest is request type for the
/// Query/GetProtoRevNumberOfTrades RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevNumberOfTradesRequest {}
impl ::prost::Name for QueryGetProtoRevNumberOfTradesRequest {
    const NAME: &'static str = "QueryGetProtoRevNumberOfTradesRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevNumberOfTradesResponse is response type for the
/// Query/GetProtoRevNumberOfTrades RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevNumberOfTradesResponse {
    /// number_of_trades is the number of trades the module has executed
    #[prost(string, tag = "1")]
    pub number_of_trades: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryGetProtoRevNumberOfTradesResponse {
    const NAME: &'static str = "QueryGetProtoRevNumberOfTradesResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevProfitsByDenomRequest is request type for the
/// Query/GetProtoRevProfitsByDenom RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevProfitsByDenomRequest {
    /// denom is the denom to query profits by
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryGetProtoRevProfitsByDenomRequest {
    const NAME: &'static str = "QueryGetProtoRevProfitsByDenomRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevProfitsByDenomResponse is response type for the
/// Query/GetProtoRevProfitsByDenom RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevProfitsByDenomResponse {
    /// profit is the profits of the module by the selected denom
    #[prost(message, optional, tag = "1")]
    pub profit: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryGetProtoRevProfitsByDenomResponse {
    const NAME: &'static str = "QueryGetProtoRevProfitsByDenomResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAllProfitsRequest is request type for the
/// Query/GetProtoRevAllProfits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAllProfitsRequest {}
impl ::prost::Name for QueryGetProtoRevAllProfitsRequest {
    const NAME: &'static str = "QueryGetProtoRevAllProfitsRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAllProfitsResponse is response type for the
/// Query/GetProtoRevAllProfits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAllProfitsResponse {
    /// profits is a list of all of the profits from the module
    #[prost(message, repeated, tag = "1")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryGetProtoRevAllProfitsResponse {
    const NAME: &'static str = "QueryGetProtoRevAllProfitsResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevStatisticsByPoolRequest is request type for the
/// Query/GetProtoRevStatisticsByRoute RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevStatisticsByRouteRequest {
    /// route is the set of pool ids to query statistics by i.e. 1,2,3
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    pub route: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for QueryGetProtoRevStatisticsByRouteRequest {
    const NAME: &'static str = "QueryGetProtoRevStatisticsByRouteRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevStatisticsByRouteResponse is response type for the
/// Query/GetProtoRevStatisticsByRoute RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevStatisticsByRouteResponse {
    /// statistics contains the number of trades the module has executed after a
    /// swap on a given pool and the profits from the trades
    #[prost(message, optional, tag = "1")]
    pub statistics: ::core::option::Option<RouteStatistics>,
}
impl ::prost::Name for QueryGetProtoRevStatisticsByRouteResponse {
    const NAME: &'static str = "QueryGetProtoRevStatisticsByRouteResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAllRouteStatisticsRequest is request type for the
/// Query/GetProtoRevAllRouteStatistics RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAllRouteStatisticsRequest {}
impl ::prost::Name for QueryGetProtoRevAllRouteStatisticsRequest {
    const NAME: &'static str = "QueryGetProtoRevAllRouteStatisticsRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAllRouteStatisticsResponse is response type for the
/// Query/GetProtoRevAllRouteStatistics RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAllRouteStatisticsResponse {
    /// statistics contains the number of trades/profits the module has executed on
    /// all routes it has successfully executed a trade on
    #[prost(message, repeated, tag = "1")]
    pub statistics: ::prost::alloc::vec::Vec<RouteStatistics>,
}
impl ::prost::Name for QueryGetProtoRevAllRouteStatisticsResponse {
    const NAME: &'static str = "QueryGetProtoRevAllRouteStatisticsResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevTokenPairArbRoutesRequest is request type for the
/// Query/GetProtoRevTokenPairArbRoutes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevTokenPairArbRoutesRequest {}
impl ::prost::Name for QueryGetProtoRevTokenPairArbRoutesRequest {
    const NAME: &'static str = "QueryGetProtoRevTokenPairArbRoutesRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevTokenPairArbRoutesResponse is response type for the
/// Query/GetProtoRevTokenPairArbRoutes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevTokenPairArbRoutesResponse {
    /// routes is a list of all of the hot routes that the module is currently
    /// arbitraging
    #[prost(message, repeated, tag = "1")]
    pub routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
}
impl ::prost::Name for QueryGetProtoRevTokenPairArbRoutesResponse {
    const NAME: &'static str = "QueryGetProtoRevTokenPairArbRoutesResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAdminAccountRequest is request type for the
/// Query/GetProtoRevAdminAccount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAdminAccountRequest {}
impl ::prost::Name for QueryGetProtoRevAdminAccountRequest {
    const NAME: &'static str = "QueryGetProtoRevAdminAccountRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevAdminAccountResponse is response type for the
/// Query/GetProtoRevAdminAccount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevAdminAccountResponse {
    /// admin_account is the admin account of the module
    #[prost(string, tag = "1")]
    pub admin_account: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryGetProtoRevAdminAccountResponse {
    const NAME: &'static str = "QueryGetProtoRevAdminAccountResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevDeveloperAccountRequest is request type for the
/// Query/GetProtoRevDeveloperAccount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevDeveloperAccountRequest {}
impl ::prost::Name for QueryGetProtoRevDeveloperAccountRequest {
    const NAME: &'static str = "QueryGetProtoRevDeveloperAccountRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevDeveloperAccountResponse is response type for the
/// Query/GetProtoRevDeveloperAccount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevDeveloperAccountResponse {
    /// developer_account is the developer account of the module
    #[prost(string, tag = "1")]
    pub developer_account: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryGetProtoRevDeveloperAccountResponse {
    const NAME: &'static str = "QueryGetProtoRevDeveloperAccountResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevInfoByPoolTypeRequest is request type for the
/// Query/GetProtoRevInfoByPoolType RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevInfoByPoolTypeRequest {}
impl ::prost::Name for QueryGetProtoRevInfoByPoolTypeRequest {
    const NAME: &'static str = "QueryGetProtoRevInfoByPoolTypeRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevInfoByPoolTypeResponse is response type for the
/// Query/GetProtoRevInfoByPoolType RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevInfoByPoolTypeResponse {
    /// InfoByPoolType contains all information pertaining to how different
    /// pool types are handled by the module.
    #[prost(message, optional, tag = "1")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
}
impl ::prost::Name for QueryGetProtoRevInfoByPoolTypeResponse {
    const NAME: &'static str = "QueryGetProtoRevInfoByPoolTypeResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevMaxPoolPointsPerBlockRequest is request type for the
/// Query/GetProtoRevMaxPoolPointsPerBlock RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevMaxPoolPointsPerBlockRequest {}
impl ::prost::Name for QueryGetProtoRevMaxPoolPointsPerBlockRequest {
    const NAME: &'static str = "QueryGetProtoRevMaxPoolPointsPerBlockRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevMaxPoolPointsPerBlockResponse is response type for the
/// Query/GetProtoRevMaxPoolPointsPerBlock RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevMaxPoolPointsPerBlockResponse {
    /// max_pool_points_per_block is the maximum number of pool points that can be
    /// consumed per block
    #[prost(uint64, tag = "1")]
    pub max_pool_points_per_block: u64,
}
impl ::prost::Name for QueryGetProtoRevMaxPoolPointsPerBlockResponse {
    const NAME: &'static str = "QueryGetProtoRevMaxPoolPointsPerBlockResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevMaxPoolPointsPerTxRequest is request type for the
/// Query/GetProtoRevMaxPoolPointsPerTx RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevMaxPoolPointsPerTxRequest {}
impl ::prost::Name for QueryGetProtoRevMaxPoolPointsPerTxRequest {
    const NAME: &'static str = "QueryGetProtoRevMaxPoolPointsPerTxRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevMaxPoolPointsPerTxResponse is response type for the
/// Query/GetProtoRevMaxPoolPointsPerTx RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevMaxPoolPointsPerTxResponse {
    /// max_pool_points_per_tx is the maximum number of pool points that can be
    /// consumed per transaction
    #[prost(uint64, tag = "1")]
    pub max_pool_points_per_tx: u64,
}
impl ::prost::Name for QueryGetProtoRevMaxPoolPointsPerTxResponse {
    const NAME: &'static str = "QueryGetProtoRevMaxPoolPointsPerTxResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevBaseDenomsRequest is request type for the
/// Query/GetProtoRevBaseDenoms RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevBaseDenomsRequest {}
impl ::prost::Name for QueryGetProtoRevBaseDenomsRequest {
    const NAME: &'static str = "QueryGetProtoRevBaseDenomsRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevBaseDenomsResponse is response type for the
/// Query/GetProtoRevBaseDenoms RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevBaseDenomsResponse {
    /// base_denoms is a list of all of the base denoms and step sizes
    #[prost(message, repeated, tag = "1")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
}
impl ::prost::Name for QueryGetProtoRevBaseDenomsResponse {
    const NAME: &'static str = "QueryGetProtoRevBaseDenomsResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevEnabledRequest is request type for the
/// Query/GetProtoRevEnabled RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevEnabledRequest {}
impl ::prost::Name for QueryGetProtoRevEnabledRequest {
    const NAME: &'static str = "QueryGetProtoRevEnabledRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevEnabledResponse is response type for the
/// Query/GetProtoRevEnabled RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevEnabledResponse {
    /// enabled is whether the module is enabled
    #[prost(bool, tag = "1")]
    pub enabled: bool,
}
impl ::prost::Name for QueryGetProtoRevEnabledResponse {
    const NAME: &'static str = "QueryGetProtoRevEnabledResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevPoolRequest is request type for the
/// Query/GetProtoRevPool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevPoolRequest {
    /// base_denom is the base denom set in protorev for the denom pair to pool
    /// mapping
    #[prost(string, tag = "1")]
    pub base_denom: ::prost::alloc::string::String,
    /// other_denom is the other denom for the denom pair to pool mapping
    #[prost(string, tag = "2")]
    pub other_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryGetProtoRevPoolRequest {
    const NAME: &'static str = "QueryGetProtoRevPoolRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// QueryGetProtoRevPoolResponse is response type for the
/// Query/GetProtoRevPool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetProtoRevPoolResponse {
    /// pool_id is the pool_id stored for the denom pair
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for QueryGetProtoRevPoolResponse {
    const NAME: &'static str = "QueryGetProtoRevPoolResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetAllProtocolRevenueRequest {}
impl ::prost::Name for QueryGetAllProtocolRevenueRequest {
    const NAME: &'static str = "QueryGetAllProtocolRevenueRequest";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGetAllProtocolRevenueResponse {
    #[prost(message, optional, tag = "1")]
    pub all_protocol_revenue: ::core::option::Option<AllProtocolRevenue>,
}
impl ::prost::Name for QueryGetAllProtocolRevenueResponse {
    const NAME: &'static str = "QueryGetAllProtocolRevenueResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetHotRoutes defines the Msg/SetHotRoutes request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetHotRoutes {
    /// admin is the account that is authorized to set the hot routes.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// hot_routes is the list of hot routes to set.
    #[prost(message, repeated, tag = "2")]
    pub hot_routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
}
impl ::prost::Name for MsgSetHotRoutes {
    const NAME: &'static str = "MsgSetHotRoutes";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetHotRoutesResponse defines the Msg/SetHotRoutes response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetHotRoutesResponse {}
impl ::prost::Name for MsgSetHotRoutesResponse {
    const NAME: &'static str = "MsgSetHotRoutesResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetDeveloperAccount defines the Msg/SetDeveloperAccount request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDeveloperAccount {
    /// admin is the account that is authorized to set the developer account.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// developer_account is the account that will receive a portion of the profits
    /// from the protorev module.
    #[prost(string, tag = "2")]
    pub developer_account: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetDeveloperAccount {
    const NAME: &'static str = "MsgSetDeveloperAccount";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetDeveloperAccountResponse defines the Msg/SetDeveloperAccount response
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDeveloperAccountResponse {}
impl ::prost::Name for MsgSetDeveloperAccountResponse {
    const NAME: &'static str = "MsgSetDeveloperAccountResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetInfoByPoolType defines the Msg/SetInfoByPoolType request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetInfoByPoolType {
    /// admin is the account that is authorized to set the pool weights.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// info_by_pool_type contains information about the pool types.
    #[prost(message, optional, tag = "2")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
}
impl ::prost::Name for MsgSetInfoByPoolType {
    const NAME: &'static str = "MsgSetInfoByPoolType";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetInfoByPoolTypeResponse defines the Msg/SetInfoByPoolType response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetInfoByPoolTypeResponse {}
impl ::prost::Name for MsgSetInfoByPoolTypeResponse {
    const NAME: &'static str = "MsgSetInfoByPoolTypeResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetMaxPoolPointsPerTx defines the Msg/SetMaxPoolPointsPerTx request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetMaxPoolPointsPerTx {
    /// admin is the account that is authorized to set the max pool points per tx.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// max_pool_points_per_tx is the maximum number of pool points that can be
    /// consumed per transaction.
    #[prost(uint64, tag = "2")]
    pub max_pool_points_per_tx: u64,
}
impl ::prost::Name for MsgSetMaxPoolPointsPerTx {
    const NAME: &'static str = "MsgSetMaxPoolPointsPerTx";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetMaxPoolPointsPerTxResponse defines the Msg/SetMaxPoolPointsPerTx
/// response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetMaxPoolPointsPerTxResponse {}
impl ::prost::Name for MsgSetMaxPoolPointsPerTxResponse {
    const NAME: &'static str = "MsgSetMaxPoolPointsPerTxResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetMaxPoolPointsPerBlock defines the Msg/SetMaxPoolPointsPerBlock request
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetMaxPoolPointsPerBlock {
    /// admin is the account that is authorized to set the max pool points per
    /// block.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// max_pool_points_per_block is the maximum number of pool points that can be
    /// consumed per block.
    #[prost(uint64, tag = "2")]
    pub max_pool_points_per_block: u64,
}
impl ::prost::Name for MsgSetMaxPoolPointsPerBlock {
    const NAME: &'static str = "MsgSetMaxPoolPointsPerBlock";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetMaxPoolPointsPerBlockResponse defines the
/// Msg/SetMaxPoolPointsPerBlock response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetMaxPoolPointsPerBlockResponse {}
impl ::prost::Name for MsgSetMaxPoolPointsPerBlockResponse {
    const NAME: &'static str = "MsgSetMaxPoolPointsPerBlockResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetBaseDenoms defines the Msg/SetBaseDenoms request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetBaseDenoms {
    /// admin is the account that is authorized to set the base denoms.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// base_denoms is the list of base denoms to set.
    #[prost(message, repeated, tag = "2")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
}
impl ::prost::Name for MsgSetBaseDenoms {
    const NAME: &'static str = "MsgSetBaseDenoms";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// Deprecated, but must be retained in the file to allow indexers
/// to index blocks since genesis
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetBaseDenomsResponse {}
impl ::prost::Name for MsgSetBaseDenomsResponse {
    const NAME: &'static str = "MsgSetBaseDenomsResponse";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetPoolWeights defines the Msg/SetPoolWeights request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetPoolWeights {
    /// admin is the account that is authorized to set the pool weights.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// pool_weights is the list of pool weights to set.
    #[prost(message, optional, tag = "2")]
    pub pool_weights: ::core::option::Option<PoolWeights>,
}
impl ::prost::Name for MsgSetPoolWeights {
    const NAME: &'static str = "MsgSetPoolWeights";
    const PACKAGE: &'static str = "osmosis.protorev.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.protorev.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.protorev.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
