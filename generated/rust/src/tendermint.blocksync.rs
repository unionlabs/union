// @generated
/// BlockRequest requests a block for a specific height
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
/// NoBlockResponse informs the node that the peer does not have block at the requested height
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoBlockResponse {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
/// BlockResponse returns block to the requested
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockResponse {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<super::types::Block>,
}
/// StatusRequest requests the status of a peer.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusRequest {}
/// StatusResponse is a peer response to inform their status.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int64, tag = "2")]
    pub base: i64,
}
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[cfg_attr(
        feature = "ethers",
        derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
    )]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        BlockRequest(super::BlockRequest),
        #[prost(message, tag = "2")]
        NoBlockResponse(super::NoBlockResponse),
        #[prost(message, tag = "3")]
        BlockResponse(super::BlockResponse),
        #[prost(message, tag = "4")]
        StatusRequest(super::StatusRequest),
        #[prost(message, tag = "5")]
        StatusResponse(super::StatusResponse),
    }
}
// @@protoc_insertion_point(module)
