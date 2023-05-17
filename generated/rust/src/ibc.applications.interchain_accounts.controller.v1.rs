// @generated
/// Params defines the set of on-chain interchain accounts parameters.
/// The following parameters may be used to disable the controller submodule.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// controller_enabled enables or disables the controller submodule.
    #[prost(bool, tag = "1")]
    pub controller_enabled: bool,
}
/// QueryInterchainAccountRequest is the request type for the Query/InterchainAccount RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInterchainAccountRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
}
/// QueryInterchainAccountResponse the response type for the Query/InterchainAccount RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInterchainAccountResponse {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// MsgRegisterInterchainAccount defines the payload for Msg/RegisterAccount
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterInterchainAccount {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub version: ::prost::alloc::string::String,
}
/// MsgRegisterInterchainAccountResponse defines the response for Msg/RegisterAccount
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterInterchainAccountResponse {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub port_id: ::prost::alloc::string::String,
}
/// MsgSendTx defines the payload for Msg/SendTx
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendTx {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub packet_data: ::core::option::Option<super::super::v1::InterchainAccountPacketData>,
    /// Relative timeout timestamp provided will be added to the current block time during transaction execution.
    /// The timeout timestamp must be non-zero.
    #[prost(uint64, tag = "4")]
    pub relative_timeout: u64,
}
/// MsgSendTxResponse defines the response for MsgSendTx
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendTxResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
include!("ibc.applications.interchain_accounts.controller.v1.tonic.rs");
// @@protoc_insertion_point(module)
