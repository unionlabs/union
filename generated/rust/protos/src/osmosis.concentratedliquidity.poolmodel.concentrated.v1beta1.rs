// @generated
/// ===================== MsgCreateConcentratedPool
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConcentratedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub denom1: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub tick_spacing: u64,
    #[prost(string, tag = "5")]
    pub spread_factor: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateConcentratedPool {
    const NAME: &'static str = "MsgCreateConcentratedPool";
    const PACKAGE: &'static str = "osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.{}",
            Self::NAME
        )
    }
}
/// Returns a unique poolID to identify the pool with.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConcentratedPoolResponse {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgCreateConcentratedPoolResponse {
    const NAME: &'static str = "MsgCreateConcentratedPoolResponse";
    const PACKAGE: &'static str = "osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.{}",
            Self::NAME
        )
    }
}
include!("osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
