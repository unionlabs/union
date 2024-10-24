// @generated
/// Permissions are the permissions that an account has to trip
/// or reset the circuit breaker.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Permissions {
    /// level is the level of permissions granted to this account.
    #[prost(enumeration = "permissions::Level", tag = "1")]
    pub level: i32,
    /// limit_type_urls is used with LEVEL_SOME_MSGS to limit the lists of Msg type
    /// URLs that the account can trip. It is an error to use limit_type_urls with
    /// a level other than LEVEL_SOME_MSGS.
    #[prost(string, repeated, tag = "2")]
    pub limit_type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `Permissions`.
pub mod permissions {
    /// Level is the permission level.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Level {
        /// LEVEL_NONE_UNSPECIFIED indicates that the account will have no circuit
        /// breaker permissions.
        NoneUnspecified = 0,
        /// LEVEL_SOME_MSGS indicates that the account will have permission to
        /// trip or reset the circuit breaker for some Msg type URLs. If this level
        /// is chosen, a non-empty list of Msg type URLs must be provided in
        /// limit_type_urls.
        SomeMsgs = 1,
        /// LEVEL_ALL_MSGS indicates that the account can trip or reset the circuit
        /// breaker for Msg's of all type URLs.
        AllMsgs = 2,
        /// LEVEL_SUPER_ADMIN indicates that the account can take all circuit breaker
        /// actions and can grant permissions to other accounts.
        SuperAdmin = 3,
    }
    impl Level {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Level::NoneUnspecified => "LEVEL_NONE_UNSPECIFIED",
                Level::SomeMsgs => "LEVEL_SOME_MSGS",
                Level::AllMsgs => "LEVEL_ALL_MSGS",
                Level::SuperAdmin => "LEVEL_SUPER_ADMIN",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "LEVEL_NONE_UNSPECIFIED" => Some(Self::NoneUnspecified),
                "LEVEL_SOME_MSGS" => Some(Self::SomeMsgs),
                "LEVEL_ALL_MSGS" => Some(Self::AllMsgs),
                "LEVEL_SUPER_ADMIN" => Some(Self::SuperAdmin),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for Permissions {
    const NAME: &'static str = "Permissions";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// GenesisAccountPermissions is the account permissions for the circuit breaker in genesis
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisAccountPermissions {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub permissions: ::core::option::Option<Permissions>,
}
impl ::prost::Name for GenesisAccountPermissions {
    const NAME: &'static str = "GenesisAccountPermissions";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// GenesisState is the state that must be provided at genesis.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub account_permissions: ::prost::alloc::vec::Vec<GenesisAccountPermissions>,
    #[prost(string, repeated, tag = "2")]
    pub disabled_type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountRequest {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAccountRequest {
    const NAME: &'static str = "QueryAccountRequest";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// AccountResponse is the response type for the Query/Account RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountResponse {
    #[prost(message, optional, tag = "1")]
    pub permission: ::core::option::Option<Permissions>,
}
impl ::prost::Name for AccountResponse {
    const NAME: &'static str = "AccountResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// QueryAccountsRequest is the request type for the Query/Accounts RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAccountsRequest {
    const NAME: &'static str = "QueryAccountsRequest";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// AccountsResponse is the response type for the Query/Accounts RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountsResponse {
    #[prost(message, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<GenesisAccountPermissions>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for AccountsResponse {
    const NAME: &'static str = "AccountsResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// QueryDisableListRequest is the request type for the Query/DisabledList RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDisabledListRequest {}
impl ::prost::Name for QueryDisabledListRequest {
    const NAME: &'static str = "QueryDisabledListRequest";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// DisabledListResponse is the response type for the Query/DisabledList RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisabledListResponse {
    #[prost(string, repeated, tag = "1")]
    pub disabled_list: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for DisabledListResponse {
    const NAME: &'static str = "DisabledListResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgAuthorizeCircuitBreaker defines the Msg/AuthorizeCircuitBreaker request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAuthorizeCircuitBreaker {
    /// granter is the granter of the circuit breaker permissions and must have
    /// LEVEL_SUPER_ADMIN.
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// grantee is the account authorized with the provided permissions.
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    /// permissions are the circuit breaker permissions that the grantee receives.
    /// These will overwrite any existing permissions. LEVEL_NONE_UNSPECIFIED can
    /// be specified to revoke all permissions.
    #[prost(message, optional, tag = "3")]
    pub permissions: ::core::option::Option<Permissions>,
}
impl ::prost::Name for MsgAuthorizeCircuitBreaker {
    const NAME: &'static str = "MsgAuthorizeCircuitBreaker";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgAuthorizeCircuitBreakerResponse defines the Msg/AuthorizeCircuitBreaker response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAuthorizeCircuitBreakerResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgAuthorizeCircuitBreakerResponse {
    const NAME: &'static str = "MsgAuthorizeCircuitBreakerResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgTripCircuitBreaker defines the Msg/TripCircuitBreaker request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTripCircuitBreaker {
    /// authority is the account authorized to trip the circuit breaker.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// msg_type_urls specifies a list of type URLs to immediately stop processing.
    /// IF IT IS LEFT EMPTY, ALL MSG PROCESSING WILL STOP IMMEDIATELY.
    /// This value is validated against the authority's permissions and if the
    /// authority does not have permissions to trip the specified msg type URLs
    /// (or all URLs), the operation will fail.
    #[prost(string, repeated, tag = "2")]
    pub msg_type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgTripCircuitBreaker {
    const NAME: &'static str = "MsgTripCircuitBreaker";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgTripCircuitBreakerResponse defines the Msg/TripCircuitBreaker response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTripCircuitBreakerResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgTripCircuitBreakerResponse {
    const NAME: &'static str = "MsgTripCircuitBreakerResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgResetCircuitBreaker defines the Msg/ResetCircuitBreaker request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgResetCircuitBreaker {
    /// authority is the account authorized to trip or reset the circuit breaker.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// msg_type_urls specifies a list of Msg type URLs to resume processing. If
    /// it is left empty all Msg processing for type URLs that the account is
    /// authorized to trip will resume.
    #[prost(string, repeated, tag = "3")]
    pub msg_type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgResetCircuitBreaker {
    const NAME: &'static str = "MsgResetCircuitBreaker";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
/// MsgResetCircuitBreakerResponse defines the Msg/ResetCircuitBreaker response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgResetCircuitBreakerResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgResetCircuitBreakerResponse {
    const NAME: &'static str = "MsgResetCircuitBreakerResponse";
    const PACKAGE: &'static str = "cosmos.circuit.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.circuit.v1.{}", Self::NAME)
    }
}
include!("cosmos.circuit.v1.tonic.rs");
// @@protoc_insertion_point(module)
