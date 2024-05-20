// @generated
/// Params defines the parameters for the diferredack module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, tag = "1")]
    pub fee_percentage: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the diferredack module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines the paramaters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWriteDiferredAck {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub diferred_packet_info: ::core::option::Option<DiferredPacketInfo>,
    #[prost(message, optional, tag = "3")]
    pub ack: ::core::option::Option<super::super::ibc::core::channel::v1::Acknowledgement>,
}
impl ::prost::Name for MsgWriteDiferredAck {
    const NAME: &'static str = "MsgWriteDiferredAck";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWriteDiferredAckResponse {}
impl ::prost::Name for MsgWriteDiferredAckResponse {
    const NAME: &'static str = "MsgWriteDiferredAckResponse";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiferredPacketInfo {
    #[prost(string, tag = "1")]
    pub refund_channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub refund_port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub packet_src_channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub packet_src_port_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "5")]
    pub packet_timeout_timestamp: u64,
    #[prost(string, tag = "6")]
    pub packet_timeout_height: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "7")]
    pub packet_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "8")]
    pub sequence: u64,
}
impl ::prost::Name for DiferredPacketInfo {
    const NAME: &'static str = "DiferredPacketInfo";
    const PACKAGE: &'static str = "diferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("diferredack.v1beta1.{}", Self::NAME)
    }
}
include!("diferredack.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
