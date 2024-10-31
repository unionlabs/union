// @generated
/// Params defines the parameters for the bank module.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// Deprecated: Use of SendEnabled in params is deprecated.
    /// For genesis, use the newly added send_enabled field in the genesis object.
    /// Storage, lookup, and manipulation of this information is now in the keeper.
    ///
    /// As of cosmos-sdk 0.47, this only exists for backwards compatibility of genesis files.
    #[deprecated]
    #[prost(message, repeated, tag = "1")]
    pub send_enabled: ::prost::alloc::vec::Vec<SendEnabled>,
    #[prost(bool, tag = "2")]
    pub default_send_enabled: bool,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// SendEnabled maps coin denom to a send_enabled status (whether a denom is
/// sendable).
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendEnabled {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub enabled: bool,
}
impl ::prost::Name for SendEnabled {
    const NAME: &'static str = "SendEnabled";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// Input models transaction input.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Input {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for Input {
    const NAME: &'static str = "Input";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// Output models transaction outputs.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for Output {
    const NAME: &'static str = "Output";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// Supply represents a struct that passively keeps track of the total supply
/// amounts in the network.
/// This message is deprecated now that supply is indexed by denom.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Supply {
    /// total is the total supply of coins across the network.
    #[prost(message, repeated, tag = "1")]
    pub total: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for Supply {
    const NAME: &'static str = "Supply";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// DenomUnit represents a struct that describes a given
/// denomination unit of the basic token.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomUnit {
    /// denom represents the string name of the given denom unit (e.g uatom).
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// exponent represents power of 10 exponent that one must
    /// raise the base_denom to in order to equal the given DenomUnit's denom
    /// 1 denom = 10^exponent base_denom
    /// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
    /// exponent = 6, thus: 1 atom = 10^6 uatom).
    #[prost(uint32, tag = "2")]
    pub exponent: u32,
    /// aliases is a list of string aliases for the given denom
    #[prost(string, repeated, tag = "3")]
    pub aliases: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for DenomUnit {
    const NAME: &'static str = "DenomUnit";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// Metadata represents a struct that describes
/// a basic token.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(string, tag = "1")]
    pub description: ::prost::alloc::string::String,
    /// denom_units represents the list of DenomUnit's for a given coin
    #[prost(message, repeated, tag = "2")]
    pub denom_units: ::prost::alloc::vec::Vec<DenomUnit>,
    /// base represents the base denom (should be the DenomUnit with exponent = 0).
    #[prost(string, tag = "3")]
    pub base: ::prost::alloc::string::String,
    /// display indicates the suggested denom that should be
    /// displayed in clients.
    #[prost(string, tag = "4")]
    pub display: ::prost::alloc::string::String,
    /// name defines the name of the token (eg: Cosmos Atom)
    #[prost(string, tag = "5")]
    pub name: ::prost::alloc::string::String,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    #[prost(string, tag = "6")]
    pub symbol: ::prost::alloc::string::String,
    /// URI to a document (on or off-chain) that contains additional information. Optional.
    #[prost(string, tag = "7")]
    pub uri: ::prost::alloc::string::String,
    /// URIHash is a sha256 hash of a document pointed by URI. It's used to verify that
    /// the document didn't change. Optional.
    #[prost(string, tag = "8")]
    pub uri_hash: ::prost::alloc::string::String,
}
impl ::prost::Name for Metadata {
    const NAME: &'static str = "Metadata";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// denom is the coin denom to query balances for.
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryBalanceRequest {
    const NAME: &'static str = "QueryBalanceRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceResponse {
    /// balance is the balance of the coin.
    #[prost(message, optional, tag = "1")]
    pub balance: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryBalanceResponse {
    const NAME: &'static str = "QueryBalanceResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryBalanceRequest is the request type for the Query/AllBalances RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
    /// resolve_denom is the flag to resolve the denom into a human-readable form from the metadata.
    #[prost(bool, tag = "3")]
    pub resolve_denom: bool,
}
impl ::prost::Name for QueryAllBalancesRequest {
    const NAME: &'static str = "QueryAllBalancesRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllBalancesResponse is the response type for the Query/AllBalances RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllBalancesResponse {
    /// balances is the balances of all the coins.
    #[prost(message, repeated, tag = "1")]
    pub balances: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAllBalancesResponse {
    const NAME: &'static str = "QueryAllBalancesResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySpendableBalancesRequest defines the gRPC request structure for querying
/// an account's spendable balances.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpendableBalancesRequest {
    /// address is the address to query spendable balances for.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QuerySpendableBalancesRequest {
    const NAME: &'static str = "QuerySpendableBalancesRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySpendableBalancesResponse defines the gRPC response structure for querying
/// an account's spendable balances.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpendableBalancesResponse {
    /// balances is the spendable balances of all the coins.
    #[prost(message, repeated, tag = "1")]
    pub balances: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QuerySpendableBalancesResponse {
    const NAME: &'static str = "QuerySpendableBalancesResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySpendableBalanceByDenomRequest defines the gRPC request structure for
/// querying an account's spendable balance for a specific denom.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpendableBalanceByDenomRequest {
    /// address is the address to query balances for.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// denom is the coin denom to query balances for.
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QuerySpendableBalanceByDenomRequest {
    const NAME: &'static str = "QuerySpendableBalanceByDenomRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySpendableBalanceByDenomResponse defines the gRPC response structure for
/// querying an account's spendable balance for a specific denom.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySpendableBalanceByDenomResponse {
    /// balance is the balance of the coin.
    #[prost(message, optional, tag = "1")]
    pub balance: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for QuerySpendableBalanceByDenomResponse {
    const NAME: &'static str = "QuerySpendableBalanceByDenomResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryTotalSupplyRequest is the request type for the Query/TotalSupply RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalSupplyRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryTotalSupplyRequest {
    const NAME: &'static str = "QueryTotalSupplyRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryTotalSupplyResponse is the response type for the Query/TotalSupply RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalSupplyResponse {
    /// supply is the supply of the coins
    #[prost(message, repeated, tag = "1")]
    pub supply: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryTotalSupplyResponse {
    const NAME: &'static str = "QueryTotalSupplyResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySupplyOfRequest is the request type for the Query/SupplyOf RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySupplyOfRequest {
    /// denom is the coin denom to query balances for.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QuerySupplyOfRequest {
    const NAME: &'static str = "QuerySupplyOfRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySupplyOfResponse is the response type for the Query/SupplyOf RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySupplyOfResponse {
    /// amount is the supply of the coin.
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for QuerySupplyOfResponse {
    const NAME: &'static str = "QuerySupplyOfResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest defines the request type for querying x/bank parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse defines the response type for querying x/bank parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params provides the parameters of the bank module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomsMetadataRequest is the request type for the Query/DenomsMetadata RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomsMetadataRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDenomsMetadataRequest {
    const NAME: &'static str = "QueryDenomsMetadataRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomsMetadataResponse is the response type for the Query/DenomsMetadata RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomsMetadataResponse {
    /// metadata provides the client information for all the registered tokens.
    #[prost(message, repeated, tag = "1")]
    pub metadatas: ::prost::alloc::vec::Vec<Metadata>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDenomsMetadataResponse {
    const NAME: &'static str = "QueryDenomsMetadataResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomMetadataRequest is the request type for the Query/DenomMetadata RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomMetadataRequest {
    /// denom is the coin denom to query the metadata for.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomMetadataRequest {
    const NAME: &'static str = "QueryDenomMetadataRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomMetadataResponse is the response type for the Query/DenomMetadata RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomMetadataResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<Metadata>,
}
impl ::prost::Name for QueryDenomMetadataResponse {
    const NAME: &'static str = "QueryDenomMetadataResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomMetadataByQueryStringRequest is the request type for the Query/DenomMetadata RPC method.
/// Identical with QueryDenomMetadataRequest but receives denom as query string.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomMetadataByQueryStringRequest {
    /// denom is the coin denom to query the metadata for.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomMetadataByQueryStringRequest {
    const NAME: &'static str = "QueryDenomMetadataByQueryStringRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomMetadataByQueryStringResponse is the response type for the Query/DenomMetadata RPC
/// method. Identical with QueryDenomMetadataResponse but receives denom as query string in request.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomMetadataByQueryStringResponse {
    /// metadata describes and provides all the client information for the requested token.
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<Metadata>,
}
impl ::prost::Name for QueryDenomMetadataByQueryStringResponse {
    const NAME: &'static str = "QueryDenomMetadataByQueryStringResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomOwnersRequest defines the request type for the DenomOwners RPC query,
/// which queries for a paginated set of all account holders of a particular
/// denomination.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomOwnersRequest {
    /// denom defines the coin denomination to query all account holders for.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDenomOwnersRequest {
    const NAME: &'static str = "QueryDenomOwnersRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// DenomOwner defines structure representing an account that owns or holds a
/// particular denominated token. It contains the account address and account
/// balance of the denominated token.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomOwner {
    /// address defines the address that owns a particular denomination.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// balance is the balance of the denominated coin for an account.
    #[prost(message, optional, tag = "2")]
    pub balance: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for DenomOwner {
    const NAME: &'static str = "DenomOwner";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomOwnersResponse defines the RPC response of a DenomOwners RPC query.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomOwnersResponse {
    #[prost(message, repeated, tag = "1")]
    pub denom_owners: ::prost::alloc::vec::Vec<DenomOwner>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDenomOwnersResponse {
    const NAME: &'static str = "QueryDenomOwnersResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomOwnersByQueryRequest defines the request type for the DenomOwnersByQuery RPC query,
/// which queries for a paginated set of all account holders of a particular
/// denomination.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomOwnersByQueryRequest {
    /// denom defines the coin denomination to query all account holders for.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDenomOwnersByQueryRequest {
    const NAME: &'static str = "QueryDenomOwnersByQueryRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomOwnersByQueryResponse defines the RPC response of a DenomOwnersByQuery RPC query.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomOwnersByQueryResponse {
    #[prost(message, repeated, tag = "1")]
    pub denom_owners: ::prost::alloc::vec::Vec<DenomOwner>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDenomOwnersByQueryResponse {
    const NAME: &'static str = "QueryDenomOwnersByQueryResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySendEnabledRequest defines the RPC request for looking up SendEnabled entries.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySendEnabledRequest {
    /// denoms is the specific denoms you want look up. Leave empty to get all entries.
    #[prost(string, repeated, tag = "1")]
    pub denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines an optional pagination for the request. This field is
    /// only read if the denoms field is empty.
    #[prost(message, optional, tag = "99")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QuerySendEnabledRequest {
    const NAME: &'static str = "QuerySendEnabledRequest";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// QuerySendEnabledResponse defines the RPC response of a SendEnable query.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySendEnabledResponse {
    #[prost(message, repeated, tag = "1")]
    pub send_enabled: ::prost::alloc::vec::Vec<SendEnabled>,
    /// pagination defines the pagination in the response. This field is only
    /// populated if the denoms field in the request is empty.
    #[prost(message, optional, tag = "99")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QuerySendEnabledResponse {
    const NAME: &'static str = "QuerySendEnabledResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgSend represents a message to send coins from one account to another.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSend {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgSend {
    const NAME: &'static str = "MsgSend";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgSendResponse defines the Msg/Send response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendResponse {}
impl ::prost::Name for MsgSendResponse {
    const NAME: &'static str = "MsgSendResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgMultiSend represents an arbitrary multi-in, multi-out send message.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMultiSend {
    /// Inputs, despite being `repeated`, only allows one sender input. This is
    /// checked in MsgMultiSend's ValidateBasic.
    #[prost(message, repeated, tag = "1")]
    pub inputs: ::prost::alloc::vec::Vec<Input>,
    #[prost(message, repeated, tag = "2")]
    pub outputs: ::prost::alloc::vec::Vec<Output>,
}
impl ::prost::Name for MsgMultiSend {
    const NAME: &'static str = "MsgMultiSend";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgMultiSendResponse defines the Msg/MultiSend response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMultiSendResponse {}
impl ::prost::Name for MsgMultiSendResponse {
    const NAME: &'static str = "MsgMultiSendResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/bank parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetSendEnabled is the Msg/SetSendEnabled request type.
///
/// Only entries to add/update/delete need to be included.
/// Existing SendEnabled entries that are not included in this
/// message are left unchanged.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetSendEnabled {
    /// authority is the address that controls the module.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// send_enabled is the list of entries to add or update.
    #[prost(message, repeated, tag = "2")]
    pub send_enabled: ::prost::alloc::vec::Vec<SendEnabled>,
    /// use_default_for is a list of denoms that should use the params.default_send_enabled value.
    /// Denoms listed here will have their SendEnabled entries deleted.
    /// If a denom is included that doesn't have a SendEnabled entry,
    /// it will be ignored.
    #[prost(string, repeated, tag = "3")]
    pub use_default_for: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgSetSendEnabled {
    const NAME: &'static str = "MsgSetSendEnabled";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetSendEnabledResponse defines the Msg/SetSendEnabled response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetSendEnabledResponse {}
impl ::prost::Name for MsgSetSendEnabledResponse {
    const NAME: &'static str = "MsgSetSendEnabledResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgBurn defines a message for burning coins.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurn {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgBurn {
    const NAME: &'static str = "MsgBurn";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// MsgBurnResponse defines the Msg/Burn response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurnResponse {}
impl ::prost::Name for MsgBurnResponse {
    const NAME: &'static str = "MsgBurnResponse";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the bank module's genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines all the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// balances is an array containing the balances of all the accounts.
    #[prost(message, repeated, tag = "2")]
    pub balances: ::prost::alloc::vec::Vec<Balance>,
    /// supply represents the total supply. If it is left empty, then supply will be calculated based on the provided
    /// balances. Otherwise, it will be used to validate that the sum of the balances equals this amount.
    #[prost(message, repeated, tag = "3")]
    pub supply: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// denom_metadata defines the metadata of the different coins.
    #[prost(message, repeated, tag = "4")]
    pub denom_metadata: ::prost::alloc::vec::Vec<Metadata>,
    /// send_enabled defines the denoms where send is enabled or disabled.
    #[prost(message, repeated, tag = "5")]
    pub send_enabled: ::prost::alloc::vec::Vec<SendEnabled>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// Balance defines an account address and balance pair used in the bank module's
/// genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Balance {
    /// address is the address of the balance holder.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// coins defines the different coins this balance holds.
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for Balance {
    const NAME: &'static str = "Balance";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
/// SendAuthorization allows the grantee to spend up to spend_limit coins from
/// the granter's account.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendAuthorization {
    #[prost(message, repeated, tag = "1")]
    pub spend_limit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// allow_list specifies an optional list of addresses to whom the grantee can send tokens on behalf of the
    /// granter. If omitted, any recipient is allowed.
    #[prost(string, repeated, tag = "2")]
    pub allow_list: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for SendAuthorization {
    const NAME: &'static str = "SendAuthorization";
    const PACKAGE: &'static str = "cosmos.bank.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.bank.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
