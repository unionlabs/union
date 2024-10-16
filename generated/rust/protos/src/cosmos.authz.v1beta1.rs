// @generated
/// GenericAuthorization gives the grantee unrestricted permissions to execute
/// the provided method on behalf of the granter's account.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericAuthorization {
    /// Msg, identified by it's type URL, to grant unrestricted permissions to execute
    #[prost(string, tag = "1")]
    pub msg: ::prost::alloc::string::String,
}
impl ::prost::Name for GenericAuthorization {
    const NAME: &'static str = "GenericAuthorization";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// Grant gives permissions to execute
/// the provide method with expiration time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Grant {
    #[prost(message, optional, tag = "1")]
    pub authorization: ::core::option::Option<::pbjson_types::Any>,
    /// time when the grant will expire and will be pruned. If null, then the grant
    /// doesn't have a time expiration (other conditions  in `authorization`
    /// may apply to invalidate the grant)
    #[prost(message, optional, tag = "2")]
    pub expiration: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for Grant {
    const NAME: &'static str = "Grant";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// GrantAuthorization extends a grant with both the addresses of the grantee and granter.
/// It is used in genesis.proto and query.proto
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantAuthorization {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub authorization: ::core::option::Option<::pbjson_types::Any>,
    #[prost(message, optional, tag = "4")]
    pub expiration: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for GrantAuthorization {
    const NAME: &'static str = "GrantAuthorization";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// GrantQueueItem contains the list of TypeURL of a sdk.Msg.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantQueueItem {
    /// msg_type_urls contains the list of TypeURL of a sdk.Msg.
    #[prost(string, repeated, tag = "1")]
    pub msg_type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for GrantQueueItem {
    const NAME: &'static str = "GrantQueueItem";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the authz module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub authorization: ::prost::alloc::vec::Vec<GrantAuthorization>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgGrant is a request type for Grant method. It declares authorization to the grantee
/// on behalf of the granter with the provided expiration time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgGrant {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub grant: ::core::option::Option<Grant>,
}
impl ::prost::Name for MsgGrant {
    const NAME: &'static str = "MsgGrant";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgGrantResponse defines the Msg/MsgGrant response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgGrantResponse {}
impl ::prost::Name for MsgGrantResponse {
    const NAME: &'static str = "MsgGrantResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgExec attempts to execute the provided messages using
/// authorizations granted to the grantee. Each message should have only
/// one signer corresponding to the granter of the authorization.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExec {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// Execute Msg.
    /// The x/authz will try to find a grant matching (msg.signers\[0\], grantee, MsgTypeURL(msg))
    /// triple and validate it.
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for MsgExec {
    const NAME: &'static str = "MsgExec";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgExecResponse defines the Msg/MsgExecResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecResponse {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub results: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for MsgExecResponse {
    const NAME: &'static str = "MsgExecResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgRevoke revokes any authorization with the provided sdk.Msg type on the
/// granter's account with that has been granted to the grantee.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRevoke {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub msg_type_url: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRevoke {
    const NAME: &'static str = "MsgRevoke";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// MsgRevokeResponse defines the Msg/MsgRevokeResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRevokeResponse {}
impl ::prost::Name for MsgRevokeResponse {
    const NAME: &'static str = "MsgRevokeResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGrantsRequest is the request type for the Query/Grants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGrantsRequest {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    /// Optional, msg_type_url, when set, will query only grants matching given msg type.
    #[prost(string, tag = "3")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryGrantsRequest {
    const NAME: &'static str = "QueryGrantsRequest";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGrantsResponse is the response type for the Query/Authorizations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGrantsResponse {
    /// authorizations is a list of grants granted for grantee by granter.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<Grant>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryGrantsResponse {
    const NAME: &'static str = "QueryGrantsResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGranterGrantsRequest is the request type for the Query/GranterGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGranterGrantsRequest {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryGranterGrantsRequest {
    const NAME: &'static str = "QueryGranterGrantsRequest";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGranterGrantsResponse is the response type for the Query/GranterGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGranterGrantsResponse {
    /// grants is a list of grants granted by the granter.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<GrantAuthorization>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryGranterGrantsResponse {
    const NAME: &'static str = "QueryGranterGrantsResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGranteeGrantsRequest is the request type for the Query/GranteeGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGranteeGrantsRequest {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryGranteeGrantsRequest {
    const NAME: &'static str = "QueryGranteeGrantsRequest";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// QueryGranteeGrantsResponse is the response type for the Query/GranteeGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGranteeGrantsResponse {
    /// grants is a list of grants granted to the grantee.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<GrantAuthorization>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryGranteeGrantsResponse {
    const NAME: &'static str = "QueryGranteeGrantsResponse";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// EventGrant is emitted on Msg/Grant
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventGrant {
    /// Msg type URL for which an autorization is granted
    #[prost(string, tag = "2")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// Granter account address
    #[prost(string, tag = "3")]
    pub granter: ::prost::alloc::string::String,
    /// Grantee account address
    #[prost(string, tag = "4")]
    pub grantee: ::prost::alloc::string::String,
}
impl ::prost::Name for EventGrant {
    const NAME: &'static str = "EventGrant";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
/// EventRevoke is emitted on Msg/Revoke
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventRevoke {
    /// Msg type URL for which an autorization is revoked
    #[prost(string, tag = "2")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// Granter account address
    #[prost(string, tag = "3")]
    pub granter: ::prost::alloc::string::String,
    /// Grantee account address
    #[prost(string, tag = "4")]
    pub grantee: ::prost::alloc::string::String,
}
impl ::prost::Name for EventRevoke {
    const NAME: &'static str = "EventRevoke";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.authz.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.authz.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
