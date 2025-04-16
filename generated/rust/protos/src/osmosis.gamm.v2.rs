// @generated
/// Deprecated: please use alternate in x/poolmanager
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpotPriceRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset_denom: ::prost::alloc::string::String,
    /// DEPRECATED
    #[deprecated]
    #[prost(bool, tag = "4")]
    pub with_swap_fee: bool,
}
impl ::prost::Name for QuerySpotPriceRequest {
    const NAME: &'static str = "QuerySpotPriceRequest";
    const PACKAGE: &'static str = "osmosis.gamm.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.v2.{}", Self::NAME)
    }
}
/// Deprecated: please use alternate in x/poolmanager
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpotPriceResponse {
    /// String of the Dec. Ex) 10.203uatom
    #[prost(string, tag = "1")]
    pub spot_price: ::prost::alloc::string::String,
}
impl ::prost::Name for QuerySpotPriceResponse {
    const NAME: &'static str = "QuerySpotPriceResponse";
    const PACKAGE: &'static str = "osmosis.gamm.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.gamm.v2.{}", Self::NAME)
    }
}
include!("osmosis.gamm.v2.tonic.rs");
// @@protoc_insertion_point(module)
