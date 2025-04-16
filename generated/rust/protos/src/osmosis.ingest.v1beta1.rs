// @generated
/// PoolData represents a structure encapsulating an Osmosis liquidity pool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolData {
    /// ChainModel is the chain representation model of the pool.
    #[prost(bytes = "vec", tag = "1")]
    pub chain_model: ::prost::alloc::vec::Vec<u8>,
    /// SqsModel is additional pool data used by the sidecar query server.
    #[prost(bytes = "vec", tag = "2")]
    pub sqs_model: ::prost::alloc::vec::Vec<u8>,
    /// TickModel is the tick data of a concentrated liquidity pool.
    /// This field is only valid and set for concentrated pools. It is nil
    /// otherwise.
    #[prost(bytes = "vec", tag = "3")]
    pub tick_model: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PoolData {
    const NAME: &'static str = "PoolData";
    const PACKAGE: &'static str = "osmosis.ingest.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ingest.v1beta1.{}", Self::NAME)
    }
}
/// The block process request.
/// Sends taker fees, block height and pools.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessBlockRequest {
    /// block height is the height of the block being processed.
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// taker_fees_map is the map of taker fees for the block.
    #[prost(bytes = "vec", tag = "2")]
    pub taker_fees_map: ::prost::alloc::vec::Vec<u8>,
    /// pools in the block.
    #[prost(message, repeated, tag = "3")]
    pub pools: ::prost::alloc::vec::Vec<PoolData>,
}
impl ::prost::Name for ProcessBlockRequest {
    const NAME: &'static str = "ProcessBlockRequest";
    const PACKAGE: &'static str = "osmosis.ingest.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ingest.v1beta1.{}", Self::NAME)
    }
}
/// The response after completing the block processing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessBlockReply {}
impl ::prost::Name for ProcessBlockReply {
    const NAME: &'static str = "ProcessBlockReply";
    const PACKAGE: &'static str = "osmosis.ingest.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ingest.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.ingest.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
