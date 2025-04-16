// @generated
/// ===================== ShareDenomResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShareDenomResponse {
    /// share_denom is the share denomination.
    #[prost(string, tag = "1")]
    pub share_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for ShareDenomResponse {
    const NAME: &'static str = "ShareDenomResponse";
    const PACKAGE: &'static str = "osmosis.cosmwasmpool.v1beta1.model.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.cosmwasmpool.v1beta1.model.v3.{}", Self::NAME)
    }
}
/// ===================== TotalPoolLiquidityResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalPoolLiquidityResponse {
    /// total_pool_liquidity is the total liquidity in the pool denominated in
    /// coins.
    #[prost(message, repeated, tag = "1")]
    pub total_pool_liquidity:
        ::prost::alloc::vec::Vec<super::super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for TotalPoolLiquidityResponse {
    const NAME: &'static str = "TotalPoolLiquidityResponse";
    const PACKAGE: &'static str = "osmosis.cosmwasmpool.v1beta1.model.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.cosmwasmpool.v1beta1.model.v3.{}", Self::NAME)
    }
}
/// ===================== AssetConfig
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetConfig {
    /// denom is the asset denomination.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// normalization_factor is the normalization factor for the asset.
    #[prost(string, tag = "2")]
    pub normalization_factor: ::prost::alloc::string::String,
}
impl ::prost::Name for AssetConfig {
    const NAME: &'static str = "AssetConfig";
    const PACKAGE: &'static str = "osmosis.cosmwasmpool.v1beta1.model.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.cosmwasmpool.v1beta1.model.v3.{}", Self::NAME)
    }
}
/// ===================== ListAssetConfigsResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAssetConfigsResponse {
    /// asset_configs is the list of asset configurations.
    #[prost(message, repeated, tag = "1")]
    pub asset_configs: ::prost::alloc::vec::Vec<AssetConfig>,
}
impl ::prost::Name for ListAssetConfigsResponse {
    const NAME: &'static str = "ListAssetConfigsResponse";
    const PACKAGE: &'static str = "osmosis.cosmwasmpool.v1beta1.model.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.cosmwasmpool.v1beta1.model.v3.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
