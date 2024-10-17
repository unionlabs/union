// @generated
/// GetBlockResults is a request for the BlockResults of a given height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for GetBlockResultsRequest {
    const NAME: &'static str = "GetBlockResultsRequest";
    const PACKAGE: &'static str = "cometbft.services.block_results.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.block_results.v1.{}", Self::NAME)
    }
}
/// GetBlockResultsResponse contains the block results for the given height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsResponse {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<super::super::super::abci::v1::ExecTxResult>,
    #[prost(message, repeated, tag = "3")]
    pub finalize_block_events: ::prost::alloc::vec::Vec<super::super::super::abci::v1::Event>,
    #[prost(message, repeated, tag = "4")]
    pub validator_updates: ::prost::alloc::vec::Vec<super::super::super::abci::v1::ValidatorUpdate>,
    #[prost(message, optional, tag = "5")]
    pub consensus_param_updates:
        ::core::option::Option<super::super::super::types::v1::ConsensusParams>,
    #[prost(bytes = "vec", tag = "6")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GetBlockResultsResponse {
    const NAME: &'static str = "GetBlockResultsResponse";
    const PACKAGE: &'static str = "cometbft.services.block_results.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.block_results.v1.{}", Self::NAME)
    }
}
include!("cometbft.services.block_results.v1.tonic.rs");
// @@protoc_insertion_point(module)
