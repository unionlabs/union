// @generated
/// Pairs defines a repeated slice of Pair objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pairs {
    #[prost(message, repeated, tag = "1")]
    pub pairs: ::prost::alloc::vec::Vec<Pair>,
}
impl ::prost::Name for Pairs {
    const NAME: &'static str = "Pairs";
    const PACKAGE: &'static str = "cosmos.store.internal.kv.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.internal.kv.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for Pair {
    const NAME: &'static str = "Pair";
    const PACKAGE: &'static str = "cosmos.store.internal.kv.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.internal.kv.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
