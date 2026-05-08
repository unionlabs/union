/// GenesisState defines the x/photon module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// MsgMintPhoton defines an sdk.Msg for burning atone and minting photons.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgMintPhoton {
    #[prost(string, tag = "1")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgMintPhotonResponse defines the response structure for executing a
/// MsgMintPhoton message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgMintPhotonResponse {
    #[prost(message, optional, tag = "1")]
    pub minted: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    /// conversion_rate represents the factor used to convert atone to photon.
    #[prost(string, tag = "2")]
    pub conversion_rate: ::prost::alloc::string::String,
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/gov parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParamsResponse {}
/// Params defines the parameters for the x/photon module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Params {
    /// Allow to mint photon or not
    #[prost(bool, tag = "1")]
    pub mint_disabled: bool,
    /// tx_fee_exceptions holds the msg type urls that are allowed to use some
    /// different tx fee coins than photon.
    /// A wildcard "*" can be used to allow all transactions to use any fee denom.
    #[prost(string, repeated, tag = "2")]
    pub tx_fee_exceptions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// QueryConversionRateRequest is request type for the Query/ConversionRate RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryConversionRateRequest {}
/// QueryConversionRateResponse is response type for the Query/ConversionRate RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryConversionRateResponse {
    /// conversion_rate represents the factor used to convert atone to photon.
    #[prost(string, tag = "1")]
    pub conversion_rate: ::prost::alloc::string::String,
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgMintPhoton {
    const NAME: &'static str = "MsgMintPhoton";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgMintPhotonResponse {
    const NAME: &'static str = "MsgMintPhotonResponse";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConversionRateRequest {
    const NAME: &'static str = "QueryConversionRateRequest";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConversionRateResponse {
    const NAME: &'static str = "QueryConversionRateResponse";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "atomone.photon.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.photon.v1.{}", Self::NAME)
    }
}
