// @generated
/// Plan specifies information about a planned upgrade and when it should occur.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Plan {
    /// Sets the name for the upgrade. This name will be used by the upgraded
    /// version of the software to apply any special "on-upgrade" commands during
    /// the first BeginBlock method after the upgrade is applied. It is also used
    /// to detect whether a software version can handle a given upgrade. If no
    /// upgrade handler with this name has been set in the software, it will be
    /// assumed that the software is out-of-date when the upgrade Time or Height is
    /// reached and the software will exit.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Deprecated: Time based upgrades have been deprecated. Time based upgrade logic
    /// has been removed from the SDK.
    /// If this field is not empty, an error will be thrown.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// The height at which the upgrade must be performed.
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// Any application specific upgrade info to be included on-chain
    /// such as a git commit that validators could automatically upgrade to
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    /// Deprecated: UpgradedClientState field has been deprecated. IBC upgrade logic has been
    /// moved to the IBC module in the sub module 02-client.
    /// If this field is not empty, an error will be thrown.
    #[deprecated]
    #[prost(message, optional, tag = "5")]
    pub upgraded_client_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for Plan {
    const NAME: &'static str = "Plan";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// SoftwareUpgradeProposal is a gov Content type for initiating a software
/// upgrade.
/// Deprecated: This legacy proposal is deprecated in favor of Msg-based gov
/// proposals, see MsgSoftwareUpgrade.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SoftwareUpgradeProposal {
    /// title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// plan of the proposal
    #[prost(message, optional, tag = "3")]
    pub plan: ::core::option::Option<Plan>,
}
impl ::prost::Name for SoftwareUpgradeProposal {
    const NAME: &'static str = "SoftwareUpgradeProposal";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// CancelSoftwareUpgradeProposal is a gov Content type for cancelling a software
/// upgrade.
/// Deprecated: This legacy proposal is deprecated in favor of Msg-based gov
/// proposals, see MsgCancelUpgrade.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelSoftwareUpgradeProposal {
    /// title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
impl ::prost::Name for CancelSoftwareUpgradeProposal {
    const NAME: &'static str = "CancelSoftwareUpgradeProposal";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// ModuleVersion specifies a module and its consensus version.
///
/// Since: cosmos-sdk 0.43
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleVersion {
    /// name of the app module
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// consensus version of the app module
    #[prost(uint64, tag = "2")]
    pub version: u64,
}
impl ::prost::Name for ModuleVersion {
    const NAME: &'static str = "ModuleVersion";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryCurrentPlanRequest is the request type for the Query/CurrentPlan RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCurrentPlanRequest {}
impl ::prost::Name for QueryCurrentPlanRequest {
    const NAME: &'static str = "QueryCurrentPlanRequest";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryCurrentPlanResponse is the response type for the Query/CurrentPlan RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCurrentPlanResponse {
    /// plan is the current upgrade plan.
    #[prost(message, optional, tag = "1")]
    pub plan: ::core::option::Option<Plan>,
}
impl ::prost::Name for QueryCurrentPlanResponse {
    const NAME: &'static str = "QueryCurrentPlanResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryCurrentPlanRequest is the request type for the Query/AppliedPlan RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAppliedPlanRequest {
    /// name is the name of the applied plan to query for.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAppliedPlanRequest {
    const NAME: &'static str = "QueryAppliedPlanRequest";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryAppliedPlanResponse is the response type for the Query/AppliedPlan RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAppliedPlanResponse {
    /// height is the block height at which the plan was applied.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for QueryAppliedPlanResponse {
    const NAME: &'static str = "QueryAppliedPlanResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryUpgradedConsensusStateRequest is the request type for the Query/UpgradedConsensusState
/// RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateRequest {
    /// last height of the current chain must be sent in request
    /// as this is the height under which next consensus state is stored
    #[prost(int64, tag = "1")]
    pub last_height: i64,
}
impl ::prost::Name for QueryUpgradedConsensusStateRequest {
    const NAME: &'static str = "QueryUpgradedConsensusStateRequest";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryUpgradedConsensusStateResponse is the response type for the Query/UpgradedConsensusState
/// RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateResponse {
    /// Since: cosmos-sdk 0.43
    #[prost(bytes = "vec", tag = "2")]
    pub upgraded_consensus_state: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryUpgradedConsensusStateResponse {
    const NAME: &'static str = "QueryUpgradedConsensusStateResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleVersionsRequest is the request type for the Query/ModuleVersions
/// RPC method.
///
/// Since: cosmos-sdk 0.43
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleVersionsRequest {
    /// module_name is a field to query a specific module
    /// consensus version from state. Leaving this empty will
    /// fetch the full list of module versions from state
    #[prost(string, tag = "1")]
    pub module_name: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryModuleVersionsRequest {
    const NAME: &'static str = "QueryModuleVersionsRequest";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleVersionsResponse is the response type for the Query/ModuleVersions
/// RPC method.
///
/// Since: cosmos-sdk 0.43
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleVersionsResponse {
    /// module_versions is a list of module names with their consensus versions.
    #[prost(message, repeated, tag = "1")]
    pub module_versions: ::prost::alloc::vec::Vec<ModuleVersion>,
}
impl ::prost::Name for QueryModuleVersionsResponse {
    const NAME: &'static str = "QueryModuleVersionsResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryAuthorityRequest is the request type for Query/Authority
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuthorityRequest {}
impl ::prost::Name for QueryAuthorityRequest {
    const NAME: &'static str = "QueryAuthorityRequest";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// QueryAuthorityResponse is the response type for Query/Authority
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAuthorityResponse {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAuthorityResponse {
    const NAME: &'static str = "QueryAuthorityResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// MsgSoftwareUpgrade is the Msg/SoftwareUpgrade request type.
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSoftwareUpgrade {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// plan is the upgrade plan.
    #[prost(message, optional, tag = "2")]
    pub plan: ::core::option::Option<Plan>,
}
impl ::prost::Name for MsgSoftwareUpgrade {
    const NAME: &'static str = "MsgSoftwareUpgrade";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// MsgSoftwareUpgradeResponse is the Msg/SoftwareUpgrade response type.
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSoftwareUpgradeResponse {}
impl ::prost::Name for MsgSoftwareUpgradeResponse {
    const NAME: &'static str = "MsgSoftwareUpgradeResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// MsgCancelUpgrade is the Msg/CancelUpgrade request type.
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUpgrade {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCancelUpgrade {
    const NAME: &'static str = "MsgCancelUpgrade";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
/// MsgCancelUpgradeResponse is the Msg/CancelUpgrade response type.
///
/// Since: cosmos-sdk 0.46
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUpgradeResponse {}
impl ::prost::Name for MsgCancelUpgradeResponse {
    const NAME: &'static str = "MsgCancelUpgradeResponse";
    const PACKAGE: &'static str = "cosmos.upgrade.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.upgrade.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.upgrade.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
