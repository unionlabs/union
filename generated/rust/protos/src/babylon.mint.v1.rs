// @generated
/// GenesisState defines the mint module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// BondDenom is the denomination of the token that should be minted.
    #[prost(string, tag = "2")]
    pub bond_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// Minter represents the mint state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Minter {
    /// InflationRate is the rate at which new tokens should be minted for the
    /// current year. For example if InflationRate=0.1, then 10% of the total
    /// supply will be minted over the course of the year.
    #[prost(string, tag = "1")]
    pub inflation_rate: ::prost::alloc::string::String,
    /// AnnualProvisions is the total number of tokens to be minted in the current
    /// year due to inflation.
    #[prost(string, tag = "2")]
    pub annual_provisions: ::prost::alloc::string::String,
    /// PreviousBlockTime is the timestamp of the previous block.
    #[prost(message, optional, tag = "4")]
    pub previous_block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// BondDenom is the denomination of the token that should be minted.
    #[prost(string, tag = "5")]
    pub bond_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for Minter {
    const NAME: &'static str = "Minter";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// GenesisTime contains the timestamp of the genesis block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisTime {
    /// GenesisTime is the timestamp of the genesis block.
    #[prost(message, optional, tag = "1")]
    pub genesis_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for GenesisTime {
    const NAME: &'static str = "GenesisTime";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryInflationRateRequest is the request type for the Query/InflationRate RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInflationRateRequest {}
impl ::prost::Name for QueryInflationRateRequest {
    const NAME: &'static str = "QueryInflationRateRequest";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryInflationRateResponse is the response type for the Query/InflationRate
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryInflationRateResponse {
    /// InflationRate is the current inflation rate.
    #[prost(bytes = "vec", tag = "1")]
    pub inflation_rate: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryInflationRateResponse {
    const NAME: &'static str = "QueryInflationRateResponse";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryAnnualProvisionsRequest is the request type for the
/// Query/AnnualProvisions RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAnnualProvisionsRequest {}
impl ::prost::Name for QueryAnnualProvisionsRequest {
    const NAME: &'static str = "QueryAnnualProvisionsRequest";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryAnnualProvisionsResponse is the response type for the
/// Query/AnnualProvisions RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAnnualProvisionsResponse {
    /// AnnualProvisions is the current annual provisions.
    #[prost(bytes = "vec", tag = "1")]
    pub annual_provisions: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryAnnualProvisionsResponse {
    const NAME: &'static str = "QueryAnnualProvisionsResponse";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryGenesisTimeRequest is the request type for the Query/GenesisTime RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGenesisTimeRequest {}
impl ::prost::Name for QueryGenesisTimeRequest {
    const NAME: &'static str = "QueryGenesisTimeRequest";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
/// QueryGenesisTimeResponse is the response type for the Query/GenesisTime RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryGenesisTimeResponse {
    /// GenesisTime is the timestamp associated with the first block.
    #[prost(message, optional, tag = "1")]
    pub genesis_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for QueryGenesisTimeResponse {
    const NAME: &'static str = "QueryGenesisTimeResponse";
    const PACKAGE: &'static str = "babylon.mint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.mint.v1.{}", Self::NAME)
    }
}
include!("babylon.mint.v1.tonic.rs");
// @@protoc_insertion_point(module)
