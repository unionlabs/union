/// Pairs defines a repeated slice of Pair objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pairs {
    #[prost(message, repeated, tag = "1")]
    pub pairs: ::prost::alloc::vec::Vec<Pair>,
}
/// Pair defines a key/value bytes tuple.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pair {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
