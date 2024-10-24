// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPing {}
impl ::prost::Name for RequestPing {
    const NAME: &'static str = "RequestPing";
    const PACKAGE: &'static str = "tendermint.rpc.grpc";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.rpc.grpc.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBroadcastTx {
    #[prost(bytes = "vec", tag = "1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestBroadcastTx {
    const NAME: &'static str = "RequestBroadcastTx";
    const PACKAGE: &'static str = "tendermint.rpc.grpc";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.rpc.grpc.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePing {}
impl ::prost::Name for ResponsePing {
    const NAME: &'static str = "ResponsePing";
    const PACKAGE: &'static str = "tendermint.rpc.grpc";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.rpc.grpc.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBroadcastTx {
    #[prost(message, optional, tag = "1")]
    pub check_tx: ::core::option::Option<super::super::abci::ResponseCheckTx>,
    #[prost(message, optional, tag = "2")]
    pub tx_result: ::core::option::Option<super::super::abci::ExecTxResult>,
}
impl ::prost::Name for ResponseBroadcastTx {
    const NAME: &'static str = "ResponseBroadcastTx";
    const PACKAGE: &'static str = "tendermint.rpc.grpc";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.rpc.grpc.{}", Self::NAME)
    }
}
include!("tendermint.rpc.grpc.tonic.rs");
// @@protoc_insertion_point(module)
