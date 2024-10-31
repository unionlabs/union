// @generated
/// GenesisState defines 08-wasm's keeper genesis state
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// uploaded light client wasm contracts
    #[prost(message, repeated, tag = "1")]
    pub contracts: ::prost::alloc::vec::Vec<Contract>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// Contract stores contract code
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Contract {
    /// contract byte code
    #[prost(bytes = "vec", tag = "1")]
    pub code_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Contract {
    const NAME: &'static str = "Contract";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreCode defines the request type for the StoreCode rpc.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCode {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// wasm byte code of light client contract. It can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "2")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreCode {
    const NAME: &'static str = "MsgStoreCode";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreCodeResponse defines the response type for the StoreCode rpc
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCodeResponse {
    /// checksum is the sha256 hash of the stored code
    #[prost(bytes = "vec", tag = "1")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreCodeResponse {
    const NAME: &'static str = "MsgStoreCodeResponse";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgRemoveChecksum defines the request type for the MsgRemoveChecksum rpc.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveChecksum {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// checksum is the sha256 hash to be removed from the store
    #[prost(bytes = "vec", tag = "2")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgRemoveChecksum {
    const NAME: &'static str = "MsgRemoveChecksum";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreChecksumResponse defines the response type for the StoreCode rpc
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveChecksumResponse {}
impl ::prost::Name for MsgRemoveChecksumResponse {
    const NAME: &'static str = "MsgRemoveChecksumResponse";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgMigrateContract defines the request type for the MigrateContract rpc.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateContract {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// the client id of the contract
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// checksum is the sha256 hash of the new wasm byte code for the contract
    #[prost(bytes = "vec", tag = "3")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
    /// the json encoded message to be passed to the contract on migration
    #[prost(bytes = "vec", tag = "4")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgMigrateContract {
    const NAME: &'static str = "MsgMigrateContract";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// MsgMigrateContractResponse defines the response type for the MigrateContract rpc
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateContractResponse {}
impl ::prost::Name for MsgMigrateContractResponse {
    const NAME: &'static str = "MsgMigrateContractResponse";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// QueryChecksumsRequest is the request type for the Query/Checksums RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChecksumsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryChecksumsRequest {
    const NAME: &'static str = "QueryChecksumsRequest";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// QueryChecksumsResponse is the response type for the Query/Checksums RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChecksumsResponse {
    /// checksums is a list of the hex encoded checksums of all wasm codes stored.
    #[prost(string, repeated, tag = "1")]
    pub checksums: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryChecksumsResponse {
    const NAME: &'static str = "QueryChecksumsResponse";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeRequest is the request type for the Query/Code RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeRequest {
    /// checksum is a hex encoded string of the code stored.
    #[prost(string, tag = "1")]
    pub checksum: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryCodeRequest {
    const NAME: &'static str = "QueryCodeRequest";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeResponse is the response type for the Query/Code RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryCodeResponse {
    const NAME: &'static str = "QueryCodeResponse";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// Wasm light client's Client state
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// bytes encoding the client state of the underlying light client
    /// implemented as a Wasm contract.
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub latest_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// Wasm light client's ConsensusState
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// bytes encoding the consensus state of the underlying light client
    /// implemented as a Wasm contract.
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// Wasm light client message (either header(s) or misbehaviour)
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientMessage {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ClientMessage {
    const NAME: &'static str = "ClientMessage";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
/// Checksums defines a list of all checksums that are stored
///
/// Deprecated: This message is deprecated in favor of storing the checksums
/// using a Collections.KeySet.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Checksums {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub checksums: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Checksums {
    const NAME: &'static str = "Checksums";
    const PACKAGE: &'static str = "ibc.lightclients.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.wasm.v1.{}", Self::NAME)
    }
}
include!("ibc.lightclients.wasm.v1.tonic.rs");
// @@protoc_insertion_point(module)
