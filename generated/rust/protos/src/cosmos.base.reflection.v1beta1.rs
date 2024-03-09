// @generated
/// ListAllInterfacesRequest is the request type of the ListAllInterfaces RPC.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAllInterfacesRequest {}
impl ::prost::Name for ListAllInterfacesRequest {
    const NAME: &'static str = "ListAllInterfacesRequest";
    const PACKAGE: &'static str = "cosmos.base.reflection.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.reflection.v1beta1.{}", Self::NAME)
    }
}
/// ListAllInterfacesResponse is the response type of the ListAllInterfaces RPC.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAllInterfacesResponse {
    /// interface_names is an array of all the registered interfaces.
    #[prost(string, repeated, tag = "1")]
    pub interface_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ListAllInterfacesResponse {
    const NAME: &'static str = "ListAllInterfacesResponse";
    const PACKAGE: &'static str = "cosmos.base.reflection.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.reflection.v1beta1.{}", Self::NAME)
    }
}
/// ListImplementationsRequest is the request type of the ListImplementations
/// RPC.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImplementationsRequest {
    /// interface_name defines the interface to query the implementations for.
    #[prost(string, tag = "1")]
    pub interface_name: ::prost::alloc::string::String,
}
impl ::prost::Name for ListImplementationsRequest {
    const NAME: &'static str = "ListImplementationsRequest";
    const PACKAGE: &'static str = "cosmos.base.reflection.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.reflection.v1beta1.{}", Self::NAME)
    }
}
/// ListImplementationsResponse is the response type of the ListImplementations
/// RPC.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImplementationsResponse {
    #[prost(string, repeated, tag = "1")]
    pub implementation_message_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ListImplementationsResponse {
    const NAME: &'static str = "ListImplementationsResponse";
    const PACKAGE: &'static str = "cosmos.base.reflection.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.reflection.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.base.reflection.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
