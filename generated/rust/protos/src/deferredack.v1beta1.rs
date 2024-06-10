// @generated
/// Params defines the parameters for the deferredack module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, tag = "1")]
    pub fee_percentage: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the deferredack module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines the paramaters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWriteDeferredAck {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub deferred_packet_info: ::core::option::Option<DeferredPacketInfo>,
    #[prost(bytes = "vec", tag = "3")]
    pub ack: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgWriteDeferredAck {
    const NAME: &'static str = "MsgWriteDeferredAck";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWriteDeferredAckResponse {}
impl ::prost::Name for MsgWriteDeferredAckResponse {
    const NAME: &'static str = "MsgWriteDeferredAckResponse";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeferredPacketInfo {
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
impl ::prost::Name for DeferredPacketInfo {
    const NAME: &'static str = "DeferredPacketInfo";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "deferredack.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("deferredack.v1beta1.{}", Self::NAME)
    }
}
include!("deferredack.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
