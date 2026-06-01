#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ResponseBase {
    #[prost(message, optional, tag = "1")]
    pub error: ::core::option::Option<super::super::google::protobuf::Any>,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<super::super::google::protobuf::Any>,
    #[prost(string, tag = "4")]
    pub log: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ResponseDeliverTx {
    #[prost(message, optional, tag = "1")]
    pub response_base: ::core::option::Option<ResponseBase>,
    #[prost(sint64, tag = "2")]
    pub gas_wanted: i64,
    #[prost(sint64, tag = "3")]
    pub gas_used: i64,
}
impl ::prost::Name for ResponseBase {
    const NAME: &'static str = "ResponseBase";
    const PACKAGE: &'static str = "tm2.abci";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.abci.{}", Self::NAME)
    }
}
impl ::prost::Name for ResponseDeliverTx {
    const NAME: &'static str = "ResponseDeliverTx";
    const PACKAGE: &'static str = "tm2.abci";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.abci.{}", Self::NAME)
    }
}
