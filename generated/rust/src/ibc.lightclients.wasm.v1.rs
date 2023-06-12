// @generated
/// QueryCodeIdsRequest is the request type for the Query/CodeIds RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeIdsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryCodeIdsResponse is the response type for the Query/CodeIds RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeIdsResponse {
    #[prost(string, repeated, tag = "1")]
    pub code_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
/// QueryCodeRequest is the request type for the Query/Code RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeRequest {
    #[prost(string, tag = "1")]
    pub code_id: ::prost::alloc::string::String,
}
/// QueryCodeResponse is the response type for the Query/Code RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub code: ::prost::alloc::vec::Vec<u8>,
}
/// MsgStoreCode defines the request type for the StoreCode rpc.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCode {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub code: ::prost::alloc::vec::Vec<u8>,
}
/// MsgStoreCodeResponse defines the response type for the StoreCode rpc
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCodeResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub code_id: ::prost::alloc::vec::Vec<u8>,
}
/// Wasm light client's keeper genesis state
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// uploaded light client wasm contracts
    #[prost(message, repeated, tag = "1")]
    pub contracts: ::prost::alloc::vec::Vec<GenesisContract>,
}
/// A contract's store key and code
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContract {
    /// store key of metadata without clientID-prefix
    #[prost(bytes = "vec", tag = "1")]
    pub code_id_key: ::prost::alloc::vec::Vec<u8>,
    /// metadata value
    #[prost(bytes = "vec", tag = "2")]
    pub contract_code: ::prost::alloc::vec::Vec<u8>,
}
/// Wasm light client's Client state
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub code_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub latest_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
/// Wasm light client's ConsensusState
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
/// Wasm light client Header
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
/// Wasm light client Misbehaviour
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
include!("ibc.lightclients.wasm.v1.tonic.rs");
// @@protoc_insertion_point(module)
