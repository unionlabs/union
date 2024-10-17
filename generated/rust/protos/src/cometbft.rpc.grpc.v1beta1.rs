// @generated
/// PingRequest is a request to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPing {}
impl ::prost::Name for RequestPing {
    const NAME: &'static str = "RequestPing";
    const PACKAGE: &'static str = "cometbft.rpc.grpc.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.rpc.grpc.v1beta1.{}", Self::NAME)
    }
}
/// RequestBroadcastTx is a request to broadcast the transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBroadcastTx {
    #[prost(bytes = "vec", tag = "1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestBroadcastTx {
    const NAME: &'static str = "RequestBroadcastTx";
    const PACKAGE: &'static str = "cometbft.rpc.grpc.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.rpc.grpc.v1beta1.{}", Self::NAME)
    }
}
/// PingResponse is a response to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePing {}
impl ::prost::Name for ResponsePing {
    const NAME: &'static str = "ResponsePing";
    const PACKAGE: &'static str = "cometbft.rpc.grpc.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.rpc.grpc.v1beta1.{}", Self::NAME)
    }
}
/// ResponseBroadcastTx is a response of broadcasting the transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBroadcastTx {
    #[prost(message, optional, tag = "1")]
    pub check_tx: ::core::option::Option<super::super::super::abci::v1beta1::ResponseCheckTx>,
    #[prost(message, optional, tag = "2")]
    pub deliver_tx: ::core::option::Option<super::super::super::abci::v1beta1::ResponseDeliverTx>,
}
impl ::prost::Name for ResponseBroadcastTx {
    const NAME: &'static str = "ResponseBroadcastTx";
    const PACKAGE: &'static str = "cometbft.rpc.grpc.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.rpc.grpc.v1beta1.{}", Self::NAME)
    }
}
include!("cometbft.rpc.grpc.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
