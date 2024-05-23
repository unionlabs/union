// @generated
/// ConfigRequest defines the request structure for the Config gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigRequest {}
impl ::prost::Name for ConfigRequest {
    const NAME: &'static str = "ConfigRequest";
    const PACKAGE: &'static str = "cosmos.base.node.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.node.v1beta1.{}", Self::NAME)
    }
}
/// ConfigResponse defines the response structure for the Config gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigResponse {
    #[prost(string, tag = "1")]
    pub minimum_gas_price: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub pruning_keep_recent: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub pruning_interval: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub halt_height: u64,
}
impl ::prost::Name for ConfigResponse {
    const NAME: &'static str = "ConfigResponse";
    const PACKAGE: &'static str = "cosmos.base.node.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.node.v1beta1.{}", Self::NAME)
    }
}
/// StateRequest defines the request structure for the status of a node.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusRequest {}
impl ::prost::Name for StatusRequest {
    const NAME: &'static str = "StatusRequest";
    const PACKAGE: &'static str = "cosmos.base.node.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.node.v1beta1.{}", Self::NAME)
    }
}
/// StateResponse defines the response structure for the status of a node.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    /// earliest block height available in the store
    #[prost(uint64, tag = "1")]
    pub earliest_store_height: u64,
    /// current block height
    #[prost(uint64, tag = "2")]
    pub height: u64,
    /// block height timestamp
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    /// app hash of the current block
    #[prost(bytes = "vec", tag = "4")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// validator hash provided by the consensus header
    #[prost(bytes = "vec", tag = "5")]
    pub validator_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for StatusResponse {
    const NAME: &'static str = "StatusResponse";
    const PACKAGE: &'static str = "cosmos.base.node.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.node.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.base.node.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
