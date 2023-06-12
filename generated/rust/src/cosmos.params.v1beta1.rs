// @generated
/// ParameterChangeProposal defines a proposal to change one or more parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParameterChangeProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub changes: ::prost::alloc::vec::Vec<ParamChange>,
}
/// ParamChange defines an individual parameter change, for use in
/// ParameterChangeProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamChange {
    #[prost(string, tag = "1")]
    pub subspace: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub value: ::prost::alloc::string::String,
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {
    /// subspace defines the module to query the parameter for.
    #[prost(string, tag = "1")]
    pub subspace: ::prost::alloc::string::String,
    /// key defines the key of the parameter in the subspace.
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// param defines the queried parameter.
    #[prost(message, optional, tag = "1")]
    pub param: ::core::option::Option<ParamChange>,
}
/// QuerySubspacesRequest defines a request type for querying for all registered
/// subspaces and all keys for a subspace.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySubspacesRequest {}
/// QuerySubspacesResponse defines the response types for querying for all
/// registered subspaces and all keys for a subspace.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySubspacesResponse {
    #[prost(message, repeated, tag = "1")]
    pub subspaces: ::prost::alloc::vec::Vec<Subspace>,
}
/// Subspace defines a parameter subspace name and all the keys that exist for
/// the subspace.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Subspace {
    #[prost(string, tag = "1")]
    pub subspace: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
include!("cosmos.params.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
