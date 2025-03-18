// @generated
/// GenesisState defines the crisis module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// constant_fee is the fee used to verify the invariant in the crisis
    /// module.
    #[prost(message, optional, tag = "3")]
    pub constant_fee: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.crisis.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.v1beta1.{}", Self::NAME)
    }
}
/// MsgVerifyInvariant represents a message to verify a particular invariance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVerifyInvariant {
    /// sender is the account address of private key to send coins to fee collector account.
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// name of the invariant module.
    #[prost(string, tag = "2")]
    pub invariant_module_name: ::prost::alloc::string::String,
    /// invariant_route is the msg's invariant route.
    #[prost(string, tag = "3")]
    pub invariant_route: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgVerifyInvariant {
    const NAME: &'static str = "MsgVerifyInvariant";
    const PACKAGE: &'static str = "cosmos.crisis.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.v1beta1.{}", Self::NAME)
    }
}
/// MsgVerifyInvariantResponse defines the Msg/VerifyInvariant response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVerifyInvariantResponse {}
impl ::prost::Name for MsgVerifyInvariantResponse {
    const NAME: &'static str = "MsgVerifyInvariantResponse";
    const PACKAGE: &'static str = "cosmos.crisis.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// constant_fee defines the x/crisis parameter.
    #[prost(message, optional, tag = "2")]
    pub constant_fee: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.crisis.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmos.crisis.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.crisis.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
