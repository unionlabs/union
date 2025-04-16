// @generated
/// SuperfluidAsset stores the pair of superfluid asset type and denom pair
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidAsset {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// AssetType indicates whether the superfluid asset is a native token or an lp
    /// share
    #[prost(enumeration = "SuperfluidAssetType", tag = "2")]
    pub asset_type: i32,
}
impl ::prost::Name for SuperfluidAsset {
    const NAME: &'static str = "SuperfluidAsset";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// SuperfluidIntermediaryAccount takes the role of intermediary between LP token
/// and OSMO tokens for superfluid staking. The intermediary account is the
/// actual account responsible for delegation, not the validator account itself.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidIntermediaryAccount {
    /// Denom indicates the denom of the superfluid asset.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub val_addr: ::prost::alloc::string::String,
    /// perpetual gauge for rewards distribution
    #[prost(uint64, tag = "3")]
    pub gauge_id: u64,
}
impl ::prost::Name for SuperfluidIntermediaryAccount {
    const NAME: &'static str = "SuperfluidIntermediaryAccount";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// The Osmo-Equivalent-Multiplier Record for epoch N refers to the osmo worth we
/// treat an LP share as having, for all of epoch N. Eventually this is intended
/// to be set as the Time-weighted-average-osmo-backing for the entire duration
/// of epoch N-1. (Thereby locking what's in use for epoch N as based on the
/// prior epochs rewards) However for now, this is not the TWAP but instead the
/// spot price at the boundary. For different types of assets in the future, it
/// could change.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OsmoEquivalentMultiplierRecord {
    #[prost(int64, tag = "1")]
    pub epoch_number: i64,
    /// superfluid asset denom, can be LP token or native token
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub multiplier: ::prost::alloc::string::String,
}
impl ::prost::Name for OsmoEquivalentMultiplierRecord {
    const NAME: &'static str = "OsmoEquivalentMultiplierRecord";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// SuperfluidDelegationRecord is a struct used to indicate superfluid
/// delegations of an account in the state machine in a user friendly form.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationRecord {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub delegation_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub equivalent_staked_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for SuperfluidDelegationRecord {
    const NAME: &'static str = "SuperfluidDelegationRecord";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// LockIdIntermediaryAccountConnection is a struct used to indicate the
/// relationship between the underlying lock id and superfluid delegation done
/// via lp shares.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockIdIntermediaryAccountConnection {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
    #[prost(string, tag = "2")]
    pub intermediary_account: ::prost::alloc::string::String,
}
impl ::prost::Name for LockIdIntermediaryAccountConnection {
    const NAME: &'static str = "LockIdIntermediaryAccountConnection";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpoolWhitelistedPools {
    #[prost(uint64, repeated, tag = "1")]
    pub ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for UnpoolWhitelistedPools {
    const NAME: &'static str = "UnpoolWhitelistedPools";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConcentratedPoolUserPositionRecord {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub position_id: u64,
    #[prost(uint64, tag = "3")]
    pub lock_id: u64,
    #[prost(message, optional, tag = "4")]
    pub synthetic_lock: ::core::option::Option<super::lockup::SyntheticLock>,
    #[prost(message, optional, tag = "5")]
    pub delegation_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "6")]
    pub equivalent_staked_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for ConcentratedPoolUserPositionRecord {
    const NAME: &'static str = "ConcentratedPoolUserPositionRecord";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// SuperfluidAssetType indicates whether the superfluid asset is
/// a native token, lp share of a pool, or concentrated share of a pool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SuperfluidAssetType {
    Native = 0,
    LpShare = 1,
    /// SuperfluidAssetTypeLendingShare = 3; // for now not exist
    ConcentratedShare = 2,
}
impl SuperfluidAssetType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SuperfluidAssetType::Native => "SuperfluidAssetTypeNative",
            SuperfluidAssetType::LpShare => "SuperfluidAssetTypeLPShare",
            SuperfluidAssetType::ConcentratedShare => "SuperfluidAssetTypeConcentratedShare",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SuperfluidAssetTypeNative" => Some(Self::Native),
            "SuperfluidAssetTypeLPShare" => Some(Self::LpShare),
            "SuperfluidAssetTypeConcentratedShare" => Some(Self::ConcentratedShare),
            _ => None,
        }
    }
}
/// Params holds parameters for the superfluid module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// minimum_risk_factor is to be cut on OSMO equivalent value of lp tokens for
    /// superfluid staking, default: 5%. The minimum risk factor works
    /// to counter-balance the staked amount on chain's exposure to various asset
    /// volatilities, and have base staking be 'resistant' to volatility.
    #[prost(string, tag = "1")]
    pub minimum_risk_factor: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// GenesisState defines the module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// superfluid_assets defines the registered superfluid assets that have been
    /// registered via governance.
    #[prost(message, repeated, tag = "2")]
    pub superfluid_assets: ::prost::alloc::vec::Vec<SuperfluidAsset>,
    /// osmo_equivalent_multipliers is the records of osmo equivalent amount of
    /// each superfluid registered pool, updated every epoch.
    #[prost(message, repeated, tag = "3")]
    pub osmo_equivalent_multipliers: ::prost::alloc::vec::Vec<OsmoEquivalentMultiplierRecord>,
    /// intermediary_accounts is a secondary account for superfluid staking that
    /// plays an intermediary role between validators and the delegators.
    #[prost(message, repeated, tag = "4")]
    pub intermediary_accounts: ::prost::alloc::vec::Vec<SuperfluidIntermediaryAccount>,
    #[prost(message, repeated, tag = "5")]
    pub intemediary_account_connections:
        ::prost::alloc::vec::Vec<LockIdIntermediaryAccountConnection>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetTypeRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for AssetTypeRequest {
    const NAME: &'static str = "AssetTypeRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetTypeResponse {
    #[prost(enumeration = "SuperfluidAssetType", tag = "1")]
    pub asset_type: i32,
}
impl ::prost::Name for AssetTypeResponse {
    const NAME: &'static str = "AssetTypeResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllAssetsRequest {}
impl ::prost::Name for AllAssetsRequest {
    const NAME: &'static str = "AllAssetsRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllAssetsResponse {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<SuperfluidAsset>,
}
impl ::prost::Name for AllAssetsResponse {
    const NAME: &'static str = "AllAssetsResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetMultiplierRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for AssetMultiplierRequest {
    const NAME: &'static str = "AssetMultiplierRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetMultiplierResponse {
    #[prost(message, optional, tag = "1")]
    pub osmo_equivalent_multiplier: ::core::option::Option<OsmoEquivalentMultiplierRecord>,
}
impl ::prost::Name for AssetMultiplierResponse {
    const NAME: &'static str = "AssetMultiplierResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidIntermediaryAccountInfo {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub gauge_id: u64,
    #[prost(string, tag = "4")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for SuperfluidIntermediaryAccountInfo {
    const NAME: &'static str = "SuperfluidIntermediaryAccountInfo";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllIntermediaryAccountsRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for AllIntermediaryAccountsRequest {
    const NAME: &'static str = "AllIntermediaryAccountsRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllIntermediaryAccountsResponse {
    #[prost(message, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<SuperfluidIntermediaryAccountInfo>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for AllIntermediaryAccountsResponse {
    const NAME: &'static str = "AllIntermediaryAccountsResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectedIntermediaryAccountRequest {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for ConnectedIntermediaryAccountRequest {
    const NAME: &'static str = "ConnectedIntermediaryAccountRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectedIntermediaryAccountResponse {
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<SuperfluidIntermediaryAccountInfo>,
}
impl ::prost::Name for ConnectedIntermediaryAccountResponse {
    const NAME: &'static str = "ConnectedIntermediaryAccountResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalDelegationByValidatorForDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryTotalDelegationByValidatorForDenomRequest {
    const NAME: &'static str = "QueryTotalDelegationByValidatorForDenomRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalDelegationByValidatorForDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<Delegations>,
}
impl ::prost::Name for QueryTotalDelegationByValidatorForDenomResponse {
    const NAME: &'static str = "QueryTotalDelegationByValidatorForDenomResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delegations {
    #[prost(string, tag = "1")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount_sfsd: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub osmo_equivalent: ::prost::alloc::string::String,
}
impl ::prost::Name for Delegations {
    const NAME: &'static str = "Delegations";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalSuperfluidDelegationsRequest {}
impl ::prost::Name for TotalSuperfluidDelegationsRequest {
    const NAME: &'static str = "TotalSuperfluidDelegationsRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TotalSuperfluidDelegationsResponse {
    #[prost(string, tag = "1")]
    pub total_delegations: ::prost::alloc::string::String,
}
impl ::prost::Name for TotalSuperfluidDelegationsResponse {
    const NAME: &'static str = "TotalSuperfluidDelegationsResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationAmountRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SuperfluidDelegationAmountRequest {
    const NAME: &'static str = "SuperfluidDelegationAmountRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationAmountResponse {
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for SuperfluidDelegationAmountResponse {
    const NAME: &'static str = "SuperfluidDelegationAmountResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationsByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for SuperfluidDelegationsByDelegatorRequest {
    const NAME: &'static str = "SuperfluidDelegationsByDelegatorRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationsByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "3")]
    pub total_equivalent_staked_amount:
        ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for SuperfluidDelegationsByDelegatorResponse {
    const NAME: &'static str = "SuperfluidDelegationsByDelegatorResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidUndelegationsByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SuperfluidUndelegationsByDelegatorRequest {
    const NAME: &'static str = "SuperfluidUndelegationsByDelegatorRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidUndelegationsByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub total_undelegated_coins:
        ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "3")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<super::lockup::SyntheticLock>,
}
impl ::prost::Name for SuperfluidUndelegationsByDelegatorResponse {
    const NAME: &'static str = "SuperfluidUndelegationsByDelegatorResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationsByValidatorDenomRequest {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for SuperfluidDelegationsByValidatorDenomRequest {
    const NAME: &'static str = "SuperfluidDelegationsByValidatorDenomRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuperfluidDelegationsByValidatorDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
}
impl ::prost::Name for SuperfluidDelegationsByValidatorDenomResponse {
    const NAME: &'static str = "SuperfluidDelegationsByValidatorDenomResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSuperfluidDelegatedAmountByValidatorDenomRequest {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for EstimateSuperfluidDelegatedAmountByValidatorDenomRequest {
    const NAME: &'static str = "EstimateSuperfluidDelegatedAmountByValidatorDenomRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EstimateSuperfluidDelegatedAmountByValidatorDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for EstimateSuperfluidDelegatedAmountByValidatorDenomResponse {
    const NAME: &'static str = "EstimateSuperfluidDelegatedAmountByValidatorDenomResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalDelegationByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryTotalDelegationByDelegatorRequest {
    const NAME: &'static str = "QueryTotalDelegationByDelegatorRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalDelegationByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub delegation_response:
        ::prost::alloc::vec::Vec<super::super::cosmos::staking::v1beta1::DelegationResponse>,
    #[prost(message, repeated, tag = "3")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub total_equivalent_staked_amount:
        ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryTotalDelegationByDelegatorResponse {
    const NAME: &'static str = "QueryTotalDelegationByDelegatorResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnpoolWhitelistRequest {}
impl ::prost::Name for QueryUnpoolWhitelistRequest {
    const NAME: &'static str = "QueryUnpoolWhitelistRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnpoolWhitelistResponse {
    #[prost(uint64, repeated, tag = "1")]
    pub pool_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for QueryUnpoolWhitelistResponse {
    const NAME: &'static str = "QueryUnpoolWhitelistResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserConcentratedSuperfluidPositionsDelegatedRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for UserConcentratedSuperfluidPositionsDelegatedRequest {
    const NAME: &'static str = "UserConcentratedSuperfluidPositionsDelegatedRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserConcentratedSuperfluidPositionsDelegatedResponse {
    #[prost(message, repeated, tag = "1")]
    pub cl_pool_user_position_records: ::prost::alloc::vec::Vec<ConcentratedPoolUserPositionRecord>,
}
impl ::prost::Name for UserConcentratedSuperfluidPositionsDelegatedResponse {
    const NAME: &'static str = "UserConcentratedSuperfluidPositionsDelegatedResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserConcentratedSuperfluidPositionsUndelegatingRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for UserConcentratedSuperfluidPositionsUndelegatingRequest {
    const NAME: &'static str = "UserConcentratedSuperfluidPositionsUndelegatingRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserConcentratedSuperfluidPositionsUndelegatingResponse {
    #[prost(message, repeated, tag = "1")]
    pub cl_pool_user_position_records: ::prost::alloc::vec::Vec<ConcentratedPoolUserPositionRecord>,
}
impl ::prost::Name for UserConcentratedSuperfluidPositionsUndelegatingResponse {
    const NAME: &'static str = "UserConcentratedSuperfluidPositionsUndelegatingResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// THIS QUERY IS TEMPORARY
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRestSupplyRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryRestSupplyRequest {
    const NAME: &'static str = "QueryRestSupplyRequest";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRestSupplyResponse {
    /// amount is the supply of the coin.
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryRestSupplyResponse {
    const NAME: &'static str = "QueryRestSupplyResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSuperfluidDelegate {
    const NAME: &'static str = "MsgSuperfluidDelegate";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidDelegateResponse {}
impl ::prost::Name for MsgSuperfluidDelegateResponse {
    const NAME: &'static str = "MsgSuperfluidDelegateResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
}
impl ::prost::Name for MsgSuperfluidUndelegate {
    const NAME: &'static str = "MsgSuperfluidUndelegate";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegateResponse {}
impl ::prost::Name for MsgSuperfluidUndelegateResponse {
    const NAME: &'static str = "MsgSuperfluidUndelegateResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUnbondLock {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
}
impl ::prost::Name for MsgSuperfluidUnbondLock {
    const NAME: &'static str = "MsgSuperfluidUnbondLock";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUnbondLockResponse {}
impl ::prost::Name for MsgSuperfluidUnbondLockResponse {
    const NAME: &'static str = "MsgSuperfluidUnbondLockResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegateAndUnbondLock {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
    /// Amount of unlocking coin.
    #[prost(message, optional, tag = "3")]
    pub coin: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgSuperfluidUndelegateAndUnbondLock {
    const NAME: &'static str = "MsgSuperfluidUndelegateAndUnbondLock";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegateAndUnbondLockResponse {
    /// lock id of the new lock created for the remaining amount.
    /// returns the original lockid if the unlocked amount is equal to the
    /// original lock's amount.
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for MsgSuperfluidUndelegateAndUnbondLockResponse {
    const NAME: &'static str = "MsgSuperfluidUndelegateAndUnbondLockResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// MsgLockAndSuperfluidDelegate locks coins with the unbonding period duration,
/// and then does a superfluid lock from the newly created lockup, to the
/// specified validator addr.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockAndSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgLockAndSuperfluidDelegate {
    const NAME: &'static str = "MsgLockAndSuperfluidDelegate";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockAndSuperfluidDelegateResponse {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
impl ::prost::Name for MsgLockAndSuperfluidDelegateResponse {
    const NAME: &'static str = "MsgLockAndSuperfluidDelegateResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// MsgCreateFullRangePositionAndSuperfluidDelegate creates a full range position
/// in a concentrated liquidity pool, then superfluid delegates.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateFullRangePositionAndSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgCreateFullRangePositionAndSuperfluidDelegate {
    const NAME: &'static str = "MsgCreateFullRangePositionAndSuperfluidDelegate";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateFullRangePositionAndSuperfluidDelegateResponse {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
    #[prost(uint64, tag = "2")]
    pub position_id: u64,
}
impl ::prost::Name for MsgCreateFullRangePositionAndSuperfluidDelegateResponse {
    const NAME: &'static str = "MsgCreateFullRangePositionAndSuperfluidDelegateResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// MsgUnPoolWhitelistedPool Unpools every lock the sender has, that is
/// associated with pool pool_id. If pool_id is not approved for unpooling by
/// governance, this is a no-op. Unpooling takes the locked gamm shares, and runs
/// "ExitPool" on it, to get the constituent tokens. e.g. z gamm/pool/1 tokens
/// ExitPools into constituent tokens x uatom, y uosmo. Then it creates a new
/// lock for every constituent token, with the duration associated with the lock.
/// If the lock was unbonding, the new lockup durations should be the time left
/// until unbond completion.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnPoolWhitelistedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
}
impl ::prost::Name for MsgUnPoolWhitelistedPool {
    const NAME: &'static str = "MsgUnPoolWhitelistedPool";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnPoolWhitelistedPoolResponse {
    #[prost(uint64, repeated, tag = "1")]
    pub exited_lock_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for MsgUnPoolWhitelistedPoolResponse {
    const NAME: &'static str = "MsgUnPoolWhitelistedPoolResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// =====================
/// MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    pub lock_id: i64,
    #[prost(message, optional, tag = "3")]
    pub shares_to_migrate: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    /// token_out_mins indicates minimum token to exit Balancer pool with.
    #[prost(message, repeated, tag = "4")]
    pub token_out_mins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition {
    const NAME: &'static str = "MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse {
    #[prost(string, tag = "1")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub liquidity_created: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub join_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse {
    const NAME: &'static str = "MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// ===================== MsgAddToConcentratedLiquiditySuperfluidPosition
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddToConcentratedLiquiditySuperfluidPosition {
    #[prost(uint64, tag = "1")]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub token_desired0: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub token_desired1: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgAddToConcentratedLiquiditySuperfluidPosition {
    const NAME: &'static str = "MsgAddToConcentratedLiquiditySuperfluidPosition";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddToConcentratedLiquiditySuperfluidPositionResponse {
    #[prost(uint64, tag = "1")]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount1: ::prost::alloc::string::String,
    /// new_liquidity is the final liquidity after the add.
    /// It includes the liquidity that existed before in the position
    /// and the new liquidity that was added to the position.
    #[prost(string, tag = "5")]
    pub new_liquidity: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub lock_id: u64,
}
impl ::prost::Name for MsgAddToConcentratedLiquiditySuperfluidPositionResponse {
    const NAME: &'static str = "MsgAddToConcentratedLiquiditySuperfluidPositionResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
/// ===================== MsgUnbondConvertAndStake
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnbondConvertAndStake {
    /// lock ID to convert and stake.
    /// lock id with 0 should be provided if converting liquid gamm shares to stake
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    /// validator address to delegate to.
    /// If provided empty string, we use the validators returned from
    /// valset-preference module.
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
    /// min_amt_to_stake indicates the minimum amount to stake after conversion
    #[prost(string, tag = "4")]
    pub min_amt_to_stake: ::prost::alloc::string::String,
    /// shares_to_convert indicates shares wanted to stake.
    /// Note that this field is only used for liquid(unlocked) gamm shares.
    /// For all other cases, this field would be disregarded.
    #[prost(message, optional, tag = "5")]
    pub shares_to_convert: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUnbondConvertAndStake {
    const NAME: &'static str = "MsgUnbondConvertAndStake";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnbondConvertAndStakeResponse {
    #[prost(string, tag = "1")]
    pub total_amt_staked: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUnbondConvertAndStakeResponse {
    const NAME: &'static str = "MsgUnbondConvertAndStakeResponse";
    const PACKAGE: &'static str = "osmosis.superfluid";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.{}", Self::NAME)
    }
}
include!("osmosis.superfluid.tonic.rs");
// @@protoc_insertion_point(module)
