// @generated
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// MaximumUnauthenticatedGas defines the maximum amount of gas that can be
    /// used to authenticate a transaction in ante handler without having fee payer
    /// authenticated.
    #[prost(uint64, tag = "1")]
    pub maximum_unauthenticated_gas: u64,
    /// IsSmartAccountActive defines the state of the authenticator.
    /// If set to false, the authenticator module will not be used
    /// and the classic cosmos sdk authentication will be used instead.
    #[prost(bool, tag = "2")]
    pub is_smart_account_active: bool,
    /// CircuitBreakerControllers defines list of addresses that are allowed to
    /// set is_smart_account_active without going through governance.
    #[prost(string, repeated, tag = "3")]
    pub circuit_breaker_controllers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// AccountAuthenticator represents a foundational model for all authenticators.
/// It provides extensibility by allowing concrete types to interpret and
/// validate transactions based on the encapsulated data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountAuthenticator {
    /// ID uniquely identifies the authenticator instance.
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// Type specifies the category of the AccountAuthenticator.
    /// This type information is essential for differentiating authenticators
    /// and ensuring precise data retrieval from the storage layer.
    #[prost(string, tag = "2")]
    pub r#type: ::prost::alloc::string::String,
    /// Config is a versatile field used in conjunction with the specific type of
    /// account authenticator to facilitate complex authentication processes.
    /// The interpretation of this field is overloaded, enabling multiple
    /// authenticators to utilize it for their respective purposes.
    #[prost(bytes = "vec", tag = "3")]
    pub config: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AccountAuthenticator {
    const NAME: &'static str = "AccountAuthenticator";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// AuthenticatorData represents a genesis exported account with Authenticators.
/// The address is used as the key, and the account authenticators are stored in
/// the authenticators field.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthenticatorData {
    /// address is an account address, one address can have many authenticators
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// authenticators are the account's authenticators, these can be multiple
    /// types including SignatureVerification, AllOfs, CosmWasmAuthenticators, etc
    #[prost(message, repeated, tag = "2")]
    pub authenticators: ::prost::alloc::vec::Vec<AccountAuthenticator>,
}
impl ::prost::Name for AuthenticatorData {
    const NAME: &'static str = "AuthenticatorData";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the authenticator module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params define the parameters for the authenticator module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// next_authenticator_id is the next available authenticator ID.
    #[prost(uint64, tag = "2")]
    pub next_authenticator_id: u64,
    /// authenticator_data contains the data for multiple accounts, each with their
    /// authenticators.
    #[prost(message, repeated, tag = "3")]
    pub authenticator_data: ::prost::alloc::vec::Vec<AuthenticatorData>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgGetAuthenticatorsRequest defines the Msg/GetAuthenticators request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthenticatorsRequest {
    #[prost(string, tag = "1")]
    pub account: ::prost::alloc::string::String,
}
impl ::prost::Name for GetAuthenticatorsRequest {
    const NAME: &'static str = "GetAuthenticatorsRequest";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgGetAuthenticatorsResponse defines the Msg/GetAuthenticators response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthenticatorsResponse {
    #[prost(message, repeated, tag = "1")]
    pub account_authenticators: ::prost::alloc::vec::Vec<AccountAuthenticator>,
}
impl ::prost::Name for GetAuthenticatorsResponse {
    const NAME: &'static str = "GetAuthenticatorsResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgGetAuthenticatorRequest defines the Msg/GetAuthenticator request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthenticatorRequest {
    #[prost(string, tag = "1")]
    pub account: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub authenticator_id: u64,
}
impl ::prost::Name for GetAuthenticatorRequest {
    const NAME: &'static str = "GetAuthenticatorRequest";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgGetAuthenticatorResponse defines the Msg/GetAuthenticator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthenticatorResponse {
    #[prost(message, optional, tag = "1")]
    pub account_authenticator: ::core::option::Option<AccountAuthenticator>,
}
impl ::prost::Name for GetAuthenticatorResponse {
    const NAME: &'static str = "GetAuthenticatorResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgAddAuthenticatorRequest defines the Msg/AddAuthenticator request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddAuthenticator {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub authenticator_type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgAddAuthenticator {
    const NAME: &'static str = "MsgAddAuthenticator";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgAddAuthenticatorResponse defines the Msg/AddAuthenticator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddAuthenticatorResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgAddAuthenticatorResponse {
    const NAME: &'static str = "MsgAddAuthenticatorResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgRemoveAuthenticatorRequest defines the Msg/RemoveAuthenticator request
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveAuthenticator {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
}
impl ::prost::Name for MsgRemoveAuthenticator {
    const NAME: &'static str = "MsgRemoveAuthenticator";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// MsgRemoveAuthenticatorResponse defines the Msg/RemoveAuthenticator response
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveAuthenticatorResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgRemoveAuthenticatorResponse {
    const NAME: &'static str = "MsgRemoveAuthenticatorResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetActiveState {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub active: bool,
}
impl ::prost::Name for MsgSetActiveState {
    const NAME: &'static str = "MsgSetActiveState";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetActiveStateResponse {}
impl ::prost::Name for MsgSetActiveStateResponse {
    const NAME: &'static str = "MsgSetActiveStateResponse";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
/// TxExtension allows for additional authenticator-specific data in
/// transactions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxExtension {
    /// selected_authenticators holds the authenticator_id for the chosen
    /// authenticator per message.
    #[prost(uint64, repeated, tag = "1")]
    pub selected_authenticators: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for TxExtension {
    const NAME: &'static str = "TxExtension";
    const PACKAGE: &'static str = "osmosis.smartaccount.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.smartaccount.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.smartaccount.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
