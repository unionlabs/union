// @generated
/// Txs contains a list of transaction from the mempool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Txs {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Txs {
    const NAME: &'static str = "Txs";
    const PACKAGE: &'static str = "cometbft.mempool.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.mempool.v1.{}", Self::NAME)
    }
}
/// Message is an abstract mempool message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    /// Sum of all possible messages.
    #[prost(oneof = "message::Sum", tags = "1")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        Txs(super::Txs),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "cometbft.mempool.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.mempool.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
