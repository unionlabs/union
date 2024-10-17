// @generated
/// Params defines the set of on-chain interchain accounts parameters.
/// The following parameters may be used to disable the controller submodule.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// controller_enabled enables or disables the controller submodule.
    #[prost(bool, tag = "1")]
    pub controller_enabled: bool,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// QueryInterchainAccountRequest is the request type for the Query/InterchainAccount RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInterchainAccountRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryInterchainAccountRequest {
    const NAME: &'static str = "QueryInterchainAccountRequest";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// QueryInterchainAccountResponse the response type for the Query/InterchainAccount RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInterchainAccountResponse {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryInterchainAccountResponse {
    const NAME: &'static str = "QueryInterchainAccountResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgRegisterInterchainAccount defines the payload for Msg/RegisterAccount
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterInterchainAccount {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub version: ::prost::alloc::string::String,
    #[prost(
        enumeration = "super::super::super::super::core::channel::v1::Order",
        tag = "4"
    )]
    pub ordering: i32,
}
impl ::prost::Name for MsgRegisterInterchainAccount {
    const NAME: &'static str = "MsgRegisterInterchainAccount";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgRegisterInterchainAccountResponse defines the response for Msg/RegisterAccount
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterInterchainAccountResponse {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub port_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRegisterInterchainAccountResponse {
    const NAME: &'static str = "MsgRegisterInterchainAccountResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgSendTx defines the payload for Msg/SendTx
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendTx {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub packet_data: ::core::option::Option<super::super::v1::InterchainAccountPacketData>,
    /// Relative timeout timestamp provided will be added to the current block time during transaction execution.
    /// The timeout timestamp must be non-zero.
    #[prost(uint64, tag = "4")]
    pub relative_timeout: u64,
}
impl ::prost::Name for MsgSendTx {
    const NAME: &'static str = "MsgSendTx";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgSendTxResponse defines the response for MsgSendTx
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendTxResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
impl ::prost::Name for MsgSendTxResponse {
    const NAME: &'static str = "MsgSendTxResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgUpdateParams defines the payload for Msg/UpdateParams
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// params defines the 27-interchain-accounts/controller parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
/// MsgUpdateParamsResponse defines the response for Msg/UpdateParams
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.controller.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.controller.v1.{}",
            Self::NAME
        )
    }
}
include!("ibc.applications.interchain_accounts.controller.v1.tonic.rs");
// @@protoc_insertion_point(module)
