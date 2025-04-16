// @generated
/// FeeToken is a struct that specifies a coin denom, and pool ID pair.
/// This marks the token as eligible for use as a tx fee asset in Osmosis.
/// Its price in osmo is derived through looking at the provided pool ID.
/// The pool ID must have osmo as one of its assets.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeeToken {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
}
impl ::prost::Name for FeeToken {
    const NAME: &'static str = "FeeToken";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// Params holds parameters for the txfees module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, repeated, tag = "1")]
    pub whitelisted_fee_token_setters: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the txfees module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(string, tag = "1")]
    pub basedenom: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub feetokens: ::prost::alloc::vec::Vec<FeeToken>,
    /// DEPRECATED
    #[deprecated]
    #[prost(message, optional, tag = "3")]
    pub tx_fees_tracker: ::core::option::Option<TxFeesTracker>,
    /// params is the container of txfees parameters.
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxFeesTracker {
    #[prost(message, repeated, tag = "1")]
    pub tx_fees: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(int64, tag = "2")]
    pub height_accounting_starts_from: i64,
}
impl ::prost::Name for TxFeesTracker {
    const NAME: &'static str = "TxFeesTracker";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// UpdateFeeTokenProposal is a gov Content type for adding new whitelisted fee
/// token(s). It must specify a denom along with gamm pool ID to use as a spot
/// price calculator. It can be used to add new denoms to the whitelist. It can
/// also be used to update the Pool to associate with the denom. If Pool ID is
/// set to 0, it will remove the denom from the whitelisted set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateFeeTokenProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub feetokens: ::prost::alloc::vec::Vec<FeeToken>,
}
impl ::prost::Name for UpdateFeeTokenProposal {
    const NAME: &'static str = "UpdateFeeTokenProposal";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeTokensRequest {}
impl ::prost::Name for QueryFeeTokensRequest {
    const NAME: &'static str = "QueryFeeTokensRequest";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeTokensResponse {
    #[prost(message, repeated, tag = "1")]
    pub fee_tokens: ::prost::alloc::vec::Vec<FeeToken>,
}
impl ::prost::Name for QueryFeeTokensResponse {
    const NAME: &'static str = "QueryFeeTokensResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomSpotPriceRequest defines grpc request structure for querying spot
/// price for the specified tx fee denom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomSpotPriceRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomSpotPriceRequest {
    const NAME: &'static str = "QueryDenomSpotPriceRequest";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomSpotPriceRequest defines grpc response structure for querying spot
/// price for the specified tx fee denom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomSpotPriceResponse {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub spot_price: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomSpotPriceResponse {
    const NAME: &'static str = "QueryDenomSpotPriceResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomPoolIdRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomPoolIdRequest {
    const NAME: &'static str = "QueryDenomPoolIdRequest";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomPoolIdResponse {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
}
impl ::prost::Name for QueryDenomPoolIdResponse {
    const NAME: &'static str = "QueryDenomPoolIdResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBaseDenomRequest {}
impl ::prost::Name for QueryBaseDenomRequest {
    const NAME: &'static str = "QueryBaseDenomRequest";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBaseDenomResponse {
    #[prost(string, tag = "1")]
    pub base_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryBaseDenomResponse {
    const NAME: &'static str = "QueryBaseDenomResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEipBaseFeeRequest {}
impl ::prost::Name for QueryEipBaseFeeRequest {
    const NAME: &'static str = "QueryEipBaseFeeRequest";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEipBaseFeeResponse {
    #[prost(string, tag = "1")]
    pub base_fee: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryEipBaseFeeResponse {
    const NAME: &'static str = "QueryEipBaseFeeResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
/// ===================== MsgSetFeeTokens
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetFeeTokens {
    #[prost(message, repeated, tag = "1")]
    pub fee_tokens: ::prost::alloc::vec::Vec<FeeToken>,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetFeeTokens {
    const NAME: &'static str = "MsgSetFeeTokens";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetFeeTokensResponse {}
impl ::prost::Name for MsgSetFeeTokensResponse {
    const NAME: &'static str = "MsgSetFeeTokensResponse";
    const PACKAGE: &'static str = "osmosis.txfees.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.txfees.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.txfees.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
