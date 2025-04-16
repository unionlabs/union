// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    #[prost(message, repeated, tag = "1")]
    pub children: ::prost::alloc::vec::Vec<Child>,
}
impl ::prost::Name for Node {
    const NAME: &'static str = "Node";
    const PACKAGE: &'static str = "osmosis.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.store.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Child {
    #[prost(bytes = "vec", tag = "1")]
    pub index: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub accumulation: ::prost::alloc::string::String,
}
impl ::prost::Name for Child {
    const NAME: &'static str = "Child";
    const PACKAGE: &'static str = "osmosis.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.store.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Leaf {
    #[prost(message, optional, tag = "1")]
    pub leaf: ::core::option::Option<Child>,
}
impl ::prost::Name for Leaf {
    const NAME: &'static str = "Leaf";
    const PACKAGE: &'static str = "osmosis.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.store.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
