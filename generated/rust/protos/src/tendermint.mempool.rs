// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Txs {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Txs {
    const NAME: &'static str = "Txs";
    const PACKAGE: &'static str = "tendermint.mempool";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.mempool.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        Txs(super::Txs),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "tendermint.mempool";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.mempool.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
