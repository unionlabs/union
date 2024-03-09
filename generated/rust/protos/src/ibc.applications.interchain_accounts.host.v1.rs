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
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.host.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.host.v1.{}",
            Self::NAME
        )
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.host.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.host.v1.{}",
            Self::NAME
        )
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.host.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.host.v1.{}",
            Self::NAME
        )
    }
}
/// MsgUpdateParams defines the payload for Msg/UpdateParams
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// params defines the 27-interchain-accounts/host parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.host.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.host.v1.{}",
            Self::NAME
        )
    }
}
/// MsgUpdateParamsResponse defines the response for Msg/UpdateParams
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.host.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.host.v1.{}",
            Self::NAME
        )
    }
}
include!("ibc.applications.interchain_accounts.host.v1.tonic.rs");
// @@protoc_insertion_point(module)
