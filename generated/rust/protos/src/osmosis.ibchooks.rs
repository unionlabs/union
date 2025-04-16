// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, repeated, tag = "1")]
    pub allowed_async_ack_contracts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.ibchooks";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ibchooks.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.ibchooks";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ibchooks.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEmitIbcAck {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub packet_sequence: u64,
    #[prost(string, tag = "3")]
    pub channel: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgEmitIbcAck {
    const NAME: &'static str = "MsgEmitIBCAck";
    const PACKAGE: &'static str = "osmosis.ibchooks";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ibchooks.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEmitIbcAckResponse {
    #[prost(string, tag = "1")]
    pub contract_result: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub ibc_ack: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgEmitIbcAckResponse {
    const NAME: &'static str = "MsgEmitIBCAckResponse";
    const PACKAGE: &'static str = "osmosis.ibchooks";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.ibchooks.{}", Self::NAME)
    }
}
include!("osmosis.ibchooks.tonic.rs");
// @@protoc_insertion_point(module)
