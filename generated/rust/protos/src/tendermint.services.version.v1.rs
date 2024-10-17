// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetVersionRequest {}
impl ::prost::Name for GetVersionRequest {
    const NAME: &'static str = "GetVersionRequest";
    const PACKAGE: &'static str = "tendermint.services.version.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.version.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetVersionResponse {
    /// The semantic version of the node software.
    #[prost(string, tag = "1")]
    pub node: ::prost::alloc::string::String,
    /// The version of ABCI used by the node.
    #[prost(string, tag = "2")]
    pub abci: ::prost::alloc::string::String,
    /// The version of the P2P protocol.
    #[prost(uint64, tag = "3")]
    pub p2p: u64,
    /// The version of the block protocol.
    #[prost(uint64, tag = "4")]
    pub block: u64,
}
impl ::prost::Name for GetVersionResponse {
    const NAME: &'static str = "GetVersionResponse";
    const PACKAGE: &'static str = "tendermint.services.version.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.services.version.v1.{}", Self::NAME)
    }
}
include!("tendermint.services.version.v1.tonic.rs");
// @@protoc_insertion_point(module)
