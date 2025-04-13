// @generated
/// Params defines the parameters for the module, including portions of rewards
/// distributed to each type of stakeholder. Note that sum of the portions should
/// be strictly less than 1 so that the rest will go to Comet
/// validators/delegations adapted from
/// <https://github.com/cosmos/cosmos-sdk/blob/release/v0.47.x/proto/cosmos/distribution/v1beta1/distribution.proto>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// btc_staking_portion is the portion of rewards that goes to Finality
    /// Providers/delegations NOTE: the portion of each Finality
    /// Provider/delegation is calculated by using its voting power and finality
    /// provider's commission
    #[prost(string, tag = "1")]
    pub btc_staking_portion: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// Gauge is an object that stores rewards to be distributed
/// code adapted from
/// <https://github.com/osmosis-labs/osmosis/blob/v18.0.0/proto/osmosis/incentives/gauge.proto>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Gauge {
    /// coins are coins that have been in the gauge
    /// Can have multiple coin denoms
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for Gauge {
    const NAME: &'static str = "Gauge";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// RewardGauge is an object that stores rewards distributed to a BTC staking
/// stakeholder code adapted from
/// <https://github.com/osmosis-labs/osmosis/blob/v18.0.0/proto/osmosis/incentives/gauge.proto>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewardGauge {
    /// coins are coins that have been in the gauge
    /// Can have multiple coin denoms
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// withdrawn_coins are coins that have been withdrawn by the stakeholder
    /// already
    #[prost(message, repeated, tag = "2")]
    pub withdrawn_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for RewardGauge {
    const NAME: &'static str = "RewardGauge";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// FinalityProviderHistoricalRewards represents the cumulative rewards ratio of
/// the finality provider per sat in that period. The period is ommited here and
/// should be part of the key used to store this structure. Key: Prefix +
/// Finality provider bech32 address + Period.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderHistoricalRewards {
    /// The cumulative rewards of that finality provider per sat until that period
    /// This coins will aways increase the value, never be reduced due to keep
    /// acumulation and when the cumulative rewards will be used to distribute
    /// rewards, 2 periods will be loaded, calculate the difference and multiplied
    /// by the total sat amount delegated
    /// <https://github.com/cosmos/cosmos-sdk/blob/e76102f885b71fd6e1c1efb692052173c4b3c3a3/x/distribution/keeper/delegation.go#L47>
    ///
    /// TODO(rafilx): add reference count for state prunning
    /// <https://github.com/cosmos/cosmos-sdk/blob/d9c53bfefc1e75a3c6b09065ea8b3a836cda0d18/x/distribution/types/distribution.pb.go#L98>
    #[prost(message, repeated, tag = "1")]
    pub cumulative_rewards_per_sat:
        ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for FinalityProviderHistoricalRewards {
    const NAME: &'static str = "FinalityProviderHistoricalRewards";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// FinalityProviderCurrentRewards represents the current rewards of the pool of
/// BTC delegations that delegated for this finality provider is entitled to.
/// Note: This rewards are for the BTC delegators that delegated to this FP
/// the FP itself is not the owner or can withdraw this rewards.
/// If a slash event happens with this finality provider, all the delegations
/// need to withdraw to the RewardGauge and the related scrutures should be
/// deleted. Key: Prefix + Finality provider bech32 address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderCurrentRewards {
    /// CurrentRewards is the current rewards that the finality provider have and
    /// it was not yet stored inside the FinalityProviderHistoricalRewards. Once
    /// something happens that modifies the amount of satoshis delegated to this
    /// finality provider or the delegators starting period (activation, unbonding
    /// or btc rewards withdraw) a new period must be created, accumulate this
    /// rewards to FinalityProviderHistoricalRewards with a new period and zero out
    /// the Current Rewards.
    #[prost(message, repeated, tag = "1")]
    pub current_rewards: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// Period stores the current period that serves as a reference for
    /// creating new historical rewards and correlate with
    /// BTCDelegationRewardsTracker StartPeriodCumulativeReward.
    #[prost(uint64, tag = "2")]
    pub period: u64,
    /// TotalActiveSat is the total amount of active satoshi delegated
    /// to this finality provider.
    #[prost(bytes = "vec", tag = "3")]
    pub total_active_sat: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for FinalityProviderCurrentRewards {
    const NAME: &'static str = "FinalityProviderCurrentRewards";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// BTCDelegationRewardsTracker represents the structure that holds information
/// from the last time this BTC delegator withdraw the rewards or modified his
/// active staked amount to one finality provider.
/// The finality provider address is ommitted here but should be part of the
/// key used to store this structure together with the BTC delegator address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegationRewardsTracker {
    /// StartPeriodCumulativeReward the starting period the the BTC delegator
    /// made his last withdraw of rewards or modified his active staking amount
    /// of satoshis.
    #[prost(uint64, tag = "1")]
    pub start_period_cumulative_reward: u64,
    /// TotalActiveSat is the total amount of active satoshi delegated
    /// to one specific finality provider.
    #[prost(bytes = "vec", tag = "2")]
    pub total_active_sat: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BtcDelegationRewardsTracker {
    const NAME: &'static str = "BTCDelegationRewardsTracker";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// GenesisState defines the incentive module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params the current params of the state.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// BTC staking gauge on every height
    #[prost(message, repeated, tag = "2")]
    pub btc_staking_gauges: ::prost::alloc::vec::Vec<BtcStakingGaugeEntry>,
    /// RewardGauges the reward gauge for each BTC staker and finality provider
    #[prost(message, repeated, tag = "3")]
    pub reward_gauges: ::prost::alloc::vec::Vec<RewardGaugeEntry>,
    /// Withdraw addresses of the delegators
    #[prost(message, repeated, tag = "4")]
    pub withdraw_addresses: ::prost::alloc::vec::Vec<WithdrawAddressEntry>,
    /// refundable_msg_hashes is the set of hashes of messages that can be refunded
    #[prost(string, repeated, tag = "5")]
    pub refundable_msg_hashes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// finality_providers_current_rewards are the current rewards of finality
    /// providers by addr
    #[prost(message, repeated, tag = "6")]
    pub finality_providers_current_rewards:
        ::prost::alloc::vec::Vec<FinalityProviderCurrentRewardsEntry>,
    /// finality_providers_historical_rewards are the historical rewards of
    /// finality providers by addr and period
    #[prost(message, repeated, tag = "7")]
    pub finality_providers_historical_rewards:
        ::prost::alloc::vec::Vec<FinalityProviderHistoricalRewardsEntry>,
    /// btc_delegation_rewards_trackers are the btc delegation rewards trackers
    /// stored by finality provider and delegator addresses
    #[prost(message, repeated, tag = "8")]
    pub btc_delegation_rewards_trackers: ::prost::alloc::vec::Vec<BtcDelegationRewardsTrackerEntry>,
    /// btc_delegators_to_fps are all the records of the delegators and the
    /// finality providers to which it delegated some BTC
    #[prost(message, repeated, tag = "9")]
    pub btc_delegators_to_fps: ::prost::alloc::vec::Vec<BtcDelegatorToFpEntry>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// BTCStakingGaugeEntry represents a gauge for BTC staking rewards at a specific
/// height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcStakingGaugeEntry {
    /// Block height at which this gauge is set
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The gauge object storing rewards info
    #[prost(message, optional, tag = "2")]
    pub gauge: ::core::option::Option<Gauge>,
}
impl ::prost::Name for BtcStakingGaugeEntry {
    const NAME: &'static str = "BTCStakingGaugeEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// RewardGaugeEntry represents a reward gauge for a specific stakeholder.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewardGaugeEntry {
    /// Type of stakeholder
    #[prost(enumeration = "StakeholderType", tag = "1")]
    pub stakeholder_type: i32,
    /// Address of the stakeholder
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
    /// The reward gauge object
    #[prost(message, optional, tag = "3")]
    pub reward_gauge: ::core::option::Option<RewardGauge>,
}
impl ::prost::Name for RewardGaugeEntry {
    const NAME: &'static str = "RewardGaugeEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// WithdrawAddressEntry holds the record of a withdraw address belonging to a
/// delegator address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WithdrawAddressEntry {
    /// Address of the delegator
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// Withdraw address
    #[prost(string, tag = "2")]
    pub withdraw_address: ::prost::alloc::string::String,
}
impl ::prost::Name for WithdrawAddressEntry {
    const NAME: &'static str = "WithdrawAddressEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// FinalityProviderCurrentRewardsEntry represents a finality provider
/// current rewards.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderCurrentRewardsEntry {
    /// Address of the finality provider
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// The finality provider current rewards
    #[prost(message, optional, tag = "2")]
    pub rewards: ::core::option::Option<FinalityProviderCurrentRewards>,
}
impl ::prost::Name for FinalityProviderCurrentRewardsEntry {
    const NAME: &'static str = "FinalityProviderCurrentRewardsEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// FinalityProviderHistoricalRewardsEntry represents a finality provider
/// historical rewards by address and period.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderHistoricalRewardsEntry {
    /// Address of the finality provider
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// Period of the historical reward
    #[prost(uint64, tag = "2")]
    pub period: u64,
    /// The finality provider historical rewards
    #[prost(message, optional, tag = "3")]
    pub rewards: ::core::option::Option<FinalityProviderHistoricalRewards>,
}
impl ::prost::Name for FinalityProviderHistoricalRewardsEntry {
    const NAME: &'static str = "FinalityProviderHistoricalRewardsEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// BTCDelegationRewardsTrackerEntry represents a BTC delegation
/// tracker entry based on the finality provider address, the delegator address
/// and a BTCDelegationTracker
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegationRewardsTrackerEntry {
    /// Address of the finality provider
    #[prost(string, tag = "1")]
    pub finality_provider_address: ::prost::alloc::string::String,
    /// Address of the delegator
    #[prost(string, tag = "2")]
    pub delegator_address: ::prost::alloc::string::String,
    /// BTC delegation tracking information
    #[prost(message, optional, tag = "3")]
    pub tracker: ::core::option::Option<BtcDelegationRewardsTracker>,
}
impl ::prost::Name for BtcDelegationRewardsTrackerEntry {
    const NAME: &'static str = "BTCDelegationRewardsTrackerEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// BTCDelegatorToFpEntry holds an entry of a delegator
/// and a finality provider to which it delegated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegatorToFpEntry {
    /// Address of the delegator
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// Address of the finality provider
    #[prost(string, tag = "2")]
    pub finality_provider_address: ::prost::alloc::string::String,
}
impl ::prost::Name for BtcDelegatorToFpEntry {
    const NAME: &'static str = "BTCDelegatorToFpEntry";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// StakeholderType represents the different types of stakeholders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum StakeholderType {
    /// Finality provider stakeholder type
    FinalityProvider = 0,
    /// BTC staker stakeholder type
    BtcStaker = 1,
}
impl StakeholderType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            StakeholderType::FinalityProvider => "FINALITY_PROVIDER",
            StakeholderType::BtcStaker => "BTC_STAKER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FINALITY_PROVIDER" => Some(Self::FinalityProvider),
            "BTC_STAKER" => Some(Self::BtcStaker),
            _ => None,
        }
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
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
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryRewardGaugesRequest is request type for the Query/RewardGauges RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRewardGaugesRequest {
    /// address is the address of the stakeholder in bech32 string
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryRewardGaugesRequest {
    const NAME: &'static str = "QueryRewardGaugesRequest";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// RewardGaugesResponse is an object that stores rewards distributed to a BTC
/// staking stakeholder
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewardGaugesResponse {
    /// coins are coins that have been in the gauge
    /// Can have multiple coin denoms
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// withdrawn_coins are coins that have been withdrawn by the stakeholder
    /// already
    #[prost(message, repeated, tag = "2")]
    pub withdrawn_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for RewardGaugesResponse {
    const NAME: &'static str = "RewardGaugesResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryRewardGaugesResponse is response type for the Query/RewardGauges RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRewardGaugesResponse {
    /// reward_gauges is the map of reward gauges, where key is the stakeholder
    /// type and value is the reward gauge holding all rewards for the stakeholder
    /// in that type
    #[prost(map = "string, message", tag = "1")]
    pub reward_gauges:
        ::std::collections::HashMap<::prost::alloc::string::String, RewardGaugesResponse>,
}
impl ::prost::Name for QueryRewardGaugesResponse {
    const NAME: &'static str = "QueryRewardGaugesResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryBTCStakingGaugeRequest is request type for the Query/BTCStakingGauge RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcStakingGaugeRequest {
    /// height is the queried Babylon height
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for QueryBtcStakingGaugeRequest {
    const NAME: &'static str = "QueryBTCStakingGaugeRequest";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// BTCStakingGaugeResponse is response type for the Query/BTCStakingGauge RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcStakingGaugeResponse {
    /// coins that have been in the gauge
    /// can have multiple coin denoms
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for BtcStakingGaugeResponse {
    const NAME: &'static str = "BTCStakingGaugeResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryBTCStakingGaugeResponse is response type for the Query/BTCStakingGauge
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcStakingGaugeResponse {
    /// gauge is the BTC staking gauge at the queried height
    #[prost(message, optional, tag = "1")]
    pub gauge: ::core::option::Option<BtcStakingGaugeResponse>,
}
impl ::prost::Name for QueryBtcStakingGaugeResponse {
    const NAME: &'static str = "QueryBTCStakingGaugeResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryDelegatorWithdrawAddressRequest is the request type for the
/// Query/DelegatorWithdrawAddress RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorWithdrawAddressRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegatorWithdrawAddressRequest {
    const NAME: &'static str = "QueryDelegatorWithdrawAddressRequest";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryDelegatorWithdrawAddressResponse is the response type for the
/// Query/DelegatorWithdrawAddress RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorWithdrawAddressResponse {
    /// withdraw_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub withdraw_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegatorWithdrawAddressResponse {
    const NAME: &'static str = "QueryDelegatorWithdrawAddressResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryDelegationRewardsRequest is the request type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsRequest {
    /// finality_provider_address defines the finality provider address of the
    /// delegation.
    #[prost(string, tag = "1")]
    pub finality_provider_address: ::prost::alloc::string::String,
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "2")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegationRewardsRequest {
    const NAME: &'static str = "QueryDelegationRewardsRequest";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// QueryDelegationRewardsResponse is the response type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsResponse {
    /// rewards are the delegation reward coins
    /// Can have multiple coin denoms
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryDelegationRewardsResponse {
    const NAME: &'static str = "QueryDelegationRewardsResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgWithdrawReward defines a message for withdrawing reward of a stakeholder.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawReward {
    /// type is the stakeholder type {finality_provider, btc_staker}
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    /// address is the address of the stakeholder in bech32 string
    /// signer of this msg has to be this address
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgWithdrawReward {
    const NAME: &'static str = "MsgWithdrawReward";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgWithdrawRewardResponse is the response to the MsgWithdrawReward message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawRewardResponse {
    /// coins is the withdrawed coins
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgWithdrawRewardResponse {
    const NAME: &'static str = "MsgWithdrawRewardResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message for updating incentive module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet
    /// implemented in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the incentive parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse is the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgSetWithdrawAddress sets the withdraw address
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetWithdrawAddress {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub withdraw_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetWithdrawAddress {
    const NAME: &'static str = "MsgSetWithdrawAddress";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
/// MsgSetWithdrawAddressResponse defines the Msg/SetWithdrawAddress response
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetWithdrawAddressResponse {}
impl ::prost::Name for MsgSetWithdrawAddressResponse {
    const NAME: &'static str = "MsgSetWithdrawAddressResponse";
    const PACKAGE: &'static str = "babylon.incentive";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.incentive.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
