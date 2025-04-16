// @generated
/// PoolParams defined the parameters that will be managed by the pool
/// governance in the future. This params are not managed by the chain
/// governance. Instead they will be managed by the token holders of the pool.
/// The pool's token holders are specified in future_pool_governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolParams {
    #[prost(string, tag = "1")]
    pub swap_fee: ::prost::alloc::string::String,
    /// N.B.: exit fee is disabled during pool creation in x/poolmanager. While old
    /// pools can maintain a non-zero fee. No new pool can be created with non-zero
    /// fee anymore
    #[prost(string, tag = "2")]
    pub exit_fee: ::prost::alloc::string::String,
}
impl ::prost::Name for PoolParams {
    const NAME: &'static str = "PoolParams";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
/// Pool is the stableswap Pool struct
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pool {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
    #[prost(message, optional, tag = "3")]
    pub pool_params: ::core::option::Option<PoolParams>,
    /// This string specifies who will govern the pool in the future.
    /// Valid forms of this are:
    /// {token name},{duration}
    /// {duration}
    /// where {token name} if specified is the token which determines the
    /// governor, and if not specified is the LP token for this pool.duration is
    /// a time specified as 0w,1w,2w, etc. which specifies how long the token
    /// would need to be locked up to count in governance. 0w means no lockup.
    #[prost(string, tag = "4")]
    pub future_pool_governor: ::prost::alloc::string::String,
    /// sum of all LP shares
    #[prost(message, optional, tag = "5")]
    pub total_shares:
        ::core::option::Option<super::super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// assets in the pool
    #[prost(message, repeated, tag = "6")]
    pub pool_liquidity:
        ::prost::alloc::vec::Vec<super::super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// for calculation amongst assets with different precisions
    #[prost(uint64, repeated, packed = "false", tag = "7")]
    pub scaling_factors: ::prost::alloc::vec::Vec<u64>,
    /// scaling_factor_controller is the address can adjust pool scaling factors
    #[prost(string, tag = "8")]
    pub scaling_factor_controller: ::prost::alloc::string::String,
}
impl ::prost::Name for Pool {
    const NAME: &'static str = "Pool";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgCreatePool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateStableswapPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pool_params: ::core::option::Option<PoolParams>,
    #[prost(message, repeated, tag = "3")]
    pub initial_pool_liquidity:
        ::prost::alloc::vec::Vec<super::super::super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(uint64, repeated, packed = "false", tag = "4")]
    pub scaling_factors: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag = "5")]
    pub future_pool_governor: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub scaling_factor_controller: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateStableswapPool {
    const NAME: &'static str = "MsgCreateStableswapPool";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
/// Returns a poolID with custom poolName.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateStableswapPoolResponse {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgCreateStableswapPoolResponse {
    const NAME: &'static str = "MsgCreateStableswapPoolResponse";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
/// Sender must be the pool's scaling_factor_governor in order for the tx to
/// succeed. Adjusts stableswap scaling factors.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStableSwapAdjustScalingFactors {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub scaling_factors: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for MsgStableSwapAdjustScalingFactors {
    const NAME: &'static str = "MsgStableSwapAdjustScalingFactors";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStableSwapAdjustScalingFactorsResponse {}
impl ::prost::Name for MsgStableSwapAdjustScalingFactorsResponse {
    const NAME: &'static str = "MsgStableSwapAdjustScalingFactorsResponse";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.stableswap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.stableswap.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.gamm.poolmodels.stableswap.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
