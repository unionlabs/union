// @generated
/// Params defines the set of on-chain interchain accounts parameters.
/// The following parameters may be used to disable the host submodule.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// host_enabled enables or disables the host submodule.
    #[prost(bool, tag = "1")]
    pub host_enabled: bool,
    /// allow_messages defines a list of sdk message typeURLs allowed to be executed on a host chain.
    #[prost(string, repeated, tag = "2")]
    pub allow_messages: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
include!("ibc.applications.interchain_accounts.host.v1.tonic.rs");
// @@protoc_insertion_point(module)
