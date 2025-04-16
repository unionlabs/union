// @generated
/// BasicAllowance implements Allowance with a one-time grant of coins
/// that optionally expires. The grantee can use up to SpendLimit to cover fees.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BasicAllowance {
    /// spend_limit specifies the maximum amount of coins that can be spent
    /// by this allowance and will be updated as coins are spent. If it is
    /// empty, there is no spend limit and any amount of coins can be spent.
    #[prost(message, repeated, tag = "1")]
    pub spend_limit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// expiration specifies an optional time when this allowance expires
    #[prost(message, optional, tag = "2")]
    pub expiration: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for BasicAllowance {
    const NAME: &'static str = "BasicAllowance";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// PeriodicAllowance extends Allowance to allow for both a maximum cap,
/// as well as a limit per time period.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeriodicAllowance {
    /// basic specifies a struct of `BasicAllowance`
    #[prost(message, optional, tag = "1")]
    pub basic: ::core::option::Option<BasicAllowance>,
    /// period specifies the time duration in which period_spend_limit coins can
    /// be spent before that allowance is reset
    #[prost(message, optional, tag = "2")]
    pub period: ::core::option::Option<::pbjson_types::Duration>,
    /// period_spend_limit specifies the maximum number of coins that can be spent
    /// in the period
    #[prost(message, repeated, tag = "3")]
    pub period_spend_limit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// period_can_spend is the number of coins left to be spent before the period_reset time
    #[prost(message, repeated, tag = "4")]
    pub period_can_spend: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// period_reset is the time at which this period resets and a new one begins,
    /// it is calculated from the start time of the first transaction after the
    /// last period ended
    #[prost(message, optional, tag = "5")]
    pub period_reset: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for PeriodicAllowance {
    const NAME: &'static str = "PeriodicAllowance";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// AllowedMsgAllowance creates allowance only for specified message types.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllowedMsgAllowance {
    /// allowance can be any of basic and periodic fee allowance.
    #[prost(message, optional, tag = "1")]
    pub allowance: ::core::option::Option<::pbjson_types::Any>,
    /// allowed_messages are the messages for which the grantee has the access.
    #[prost(string, repeated, tag = "2")]
    pub allowed_messages: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for AllowedMsgAllowance {
    const NAME: &'static str = "AllowedMsgAllowance";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// Grant is stored in the KVStore to record a grant with full context
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Grant {
    /// granter is the address of the user granting an allowance of their funds.
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// grantee is the address of the user being granted an allowance of another user's funds.
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    /// allowance can be any of basic, periodic, allowed fee allowance.
    #[prost(message, optional, tag = "3")]
    pub allowance: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for Grant {
    const NAME: &'static str = "Grant";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState contains a set of fee allowances, persisted from the store
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub allowances: ::prost::alloc::vec::Vec<Grant>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowanceRequest is the request type for the Query/Allowance RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowanceRequest {
    /// granter is the address of the user granting an allowance of their funds.
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// grantee is the address of the user being granted an allowance of another user's funds.
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAllowanceRequest {
    const NAME: &'static str = "QueryAllowanceRequest";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowanceResponse is the response type for the Query/Allowance RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowanceResponse {
    /// allowance is a allowance granted for grantee by granter.
    #[prost(message, optional, tag = "1")]
    pub allowance: ::core::option::Option<Grant>,
}
impl ::prost::Name for QueryAllowanceResponse {
    const NAME: &'static str = "QueryAllowanceResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowancesRequest is the request type for the Query/Allowances RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowancesRequest {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAllowancesRequest {
    const NAME: &'static str = "QueryAllowancesRequest";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowancesResponse is the response type for the Query/Allowances RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowancesResponse {
    /// allowances are allowance's granted for grantee by granter.
    #[prost(message, repeated, tag = "1")]
    pub allowances: ::prost::alloc::vec::Vec<Grant>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAllowancesResponse {
    const NAME: &'static str = "QueryAllowancesResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowancesByGranterRequest is the request type for the Query/AllowancesByGranter RPC method.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowancesByGranterRequest {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAllowancesByGranterRequest {
    const NAME: &'static str = "QueryAllowancesByGranterRequest";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllowancesByGranterResponse is the response type for the Query/AllowancesByGranter RPC method.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllowancesByGranterResponse {
    /// allowances that have been issued by the granter.
    #[prost(message, repeated, tag = "1")]
    pub allowances: ::prost::alloc::vec::Vec<Grant>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAllowancesByGranterResponse {
    const NAME: &'static str = "QueryAllowancesByGranterResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgGrantAllowance adds permission for Grantee to spend up to Allowance
/// of fees from the account of Granter.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgGrantAllowance {
    /// granter is the address of the user granting an allowance of their funds.
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// grantee is the address of the user being granted an allowance of another user's funds.
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    /// allowance can be any of basic, periodic, allowed fee allowance.
    #[prost(message, optional, tag = "3")]
    pub allowance: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for MsgGrantAllowance {
    const NAME: &'static str = "MsgGrantAllowance";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgGrantAllowanceResponse defines the Msg/GrantAllowanceResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgGrantAllowanceResponse {}
impl ::prost::Name for MsgGrantAllowanceResponse {
    const NAME: &'static str = "MsgGrantAllowanceResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgRevokeAllowance removes any existing Allowance from Granter to Grantee.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRevokeAllowance {
    /// granter is the address of the user granting an allowance of their funds.
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// grantee is the address of the user being granted an allowance of another user's funds.
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRevokeAllowance {
    const NAME: &'static str = "MsgRevokeAllowance";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgRevokeAllowanceResponse defines the Msg/RevokeAllowanceResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRevokeAllowanceResponse {}
impl ::prost::Name for MsgRevokeAllowanceResponse {
    const NAME: &'static str = "MsgRevokeAllowanceResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgPruneAllowances prunes expired fee allowances.
///
/// Since cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPruneAllowances {
    /// pruner is the address of the user pruning expired allowances.
    #[prost(string, tag = "1")]
    pub pruner: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgPruneAllowances {
    const NAME: &'static str = "MsgPruneAllowances";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
/// MsgPruneAllowancesResponse defines the Msg/PruneAllowancesResponse response type.
///
/// Since cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPruneAllowancesResponse {}
impl ::prost::Name for MsgPruneAllowancesResponse {
    const NAME: &'static str = "MsgPruneAllowancesResponse";
    const PACKAGE: &'static str = "cosmos.feegrant.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.feegrant.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.feegrant.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
