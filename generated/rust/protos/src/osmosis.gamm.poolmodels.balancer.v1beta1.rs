// @generated
/// ===================== MsgCreatePool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateBalancerPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pool_params: ::core::option::Option<super::super::super::v1beta1::PoolParams>,
    #[prost(message, repeated, tag = "3")]
    pub pool_assets: ::prost::alloc::vec::Vec<super::super::super::v1beta1::PoolAsset>,
    #[prost(string, tag = "4")]
    pub future_pool_governor: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateBalancerPool {
    const NAME: &'static str = "MsgCreateBalancerPool";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.balancer.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.balancer.v1beta1.{}", Self::NAME)
    }
}
/// Returns the poolID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateBalancerPoolResponse {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgCreateBalancerPoolResponse {
    const NAME: &'static str = "MsgCreateBalancerPoolResponse";
    const PACKAGE: &'static str = "osmosis.gamm.poolmodels.balancer.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.poolmodels.balancer.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.gamm.poolmodels.balancer.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
