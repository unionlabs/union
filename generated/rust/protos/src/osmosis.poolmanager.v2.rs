// @generated
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
    const PACKAGE: &'static str = "osmosis.poolmanager.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v2.{}", Self::NAME)
    }
}
/// SpotPriceResponse defines the gRPC response structure for a SpotPrice
/// query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotPriceResponse {
    /// String of the BigDec. Ex) 10.203uatom
    #[prost(string, tag = "1")]
    pub spot_price: ::prost::alloc::string::String,
}
impl ::prost::Name for SpotPriceResponse {
    const NAME: &'static str = "SpotPriceResponse";
    const PACKAGE: &'static str = "osmosis.poolmanager.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.poolmanager.v2.{}", Self::NAME)
    }
}
include!("osmosis.poolmanager.v2.tonic.rs");
// @@protoc_insertion_point(module)
