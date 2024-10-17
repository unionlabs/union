// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetByHeightRequest {
    /// The height of the block requested.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for GetByHeightRequest {
    const NAME: &'static str = "GetByHeightRequest";
    const PACKAGE: &'static str = "tendermint.services.block.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.block.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<super::super::super::types::BlockId>,
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<super::super::super::types::Block>,
}
impl ::prost::Name for GetByHeightResponse {
    const NAME: &'static str = "GetByHeightResponse";
    const PACKAGE: &'static str = "tendermint.services.block.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.block.v1.{}", Self::NAME)
    }
}
/// GetLatestHeightRequest - empty message since no parameter is required
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestHeightRequest {}
impl ::prost::Name for GetLatestHeightRequest {
    const NAME: &'static str = "GetLatestHeightRequest";
    const PACKAGE: &'static str = "tendermint.services.block.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.block.v1.{}", Self::NAME)
    }
}
/// GetLatestHeightResponse provides the height of the latest committed block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestHeightResponse {
    /// The height of the latest committed block. Will be 0 if no data has been
    /// committed yet.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for GetLatestHeightResponse {
    const NAME: &'static str = "GetLatestHeightResponse";
    const PACKAGE: &'static str = "tendermint.services.block.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.block.v1.{}", Self::NAME)
    }
}
include!("tendermint.services.block.v1.tonic.rs");
// @@protoc_insertion_point(module)
