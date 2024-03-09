// @generated
/// Params defines the set of params for the distribution module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, tag = "1")]
    pub community_tax: ::prost::alloc::string::String,
    /// Deprecated: The base_proposer_reward field is deprecated and is no longer used
    /// in the x/distribution module's reward mechanism.
    #[deprecated]
    #[prost(string, tag = "2")]
    pub base_proposer_reward: ::prost::alloc::string::String,
    /// Deprecated: The bonus_proposer_reward field is deprecated and is no longer used
    /// in the x/distribution module's reward mechanism.
    #[deprecated]
    #[prost(string, tag = "3")]
    pub bonus_proposer_reward: ::prost::alloc::string::String,
    #[prost(bool, tag = "4")]
    pub withdraw_addr_enabled: bool,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorHistoricalRewards represents historical rewards for a validator.
/// Height is implicit within the store key.
/// Cumulative reward ratio is the sum from the zeroeth period
/// until this period of rewards / tokens, per the spec.
/// The reference count indicates the number of objects
/// which might need to reference this historical entry at any point.
/// ReferenceCount =
///     number of outstanding delegations which ended the associated period (and
///     might need to read that record)
///   + number of slashes which ended the associated period (and might need to
///   read that record)
///   + one per validator for the zeroeth period, set on initialization
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorHistoricalRewards {
    #[prost(message, repeated, tag = "1")]
    pub cumulative_reward_ratio: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    #[prost(uint32, tag = "2")]
    pub reference_count: u32,
}
impl ::prost::Name for ValidatorHistoricalRewards {
    const NAME: &'static str = "ValidatorHistoricalRewards";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorCurrentRewards represents current rewards and current
/// period for a validator kept as a running counter and incremented
/// each block as long as the validator's tokens remain constant.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorCurrentRewards {
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    #[prost(uint64, tag = "2")]
    pub period: u64,
}
impl ::prost::Name for ValidatorCurrentRewards {
    const NAME: &'static str = "ValidatorCurrentRewards";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorAccumulatedCommission represents accumulated commission
/// for a validator kept as a running counter, can be withdrawn at any time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorAccumulatedCommission {
    #[prost(message, repeated, tag = "1")]
    pub commission: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for ValidatorAccumulatedCommission {
    const NAME: &'static str = "ValidatorAccumulatedCommission";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorOutstandingRewards represents outstanding (un-withdrawn) rewards
/// for a validator inexpensive to track, allows simple sanity checks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorOutstandingRewards {
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for ValidatorOutstandingRewards {
    const NAME: &'static str = "ValidatorOutstandingRewards";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorSlashEvent represents a validator slash event.
/// Height is implicit within the store key.
/// This is needed to calculate appropriate amount of staking tokens
/// for delegations which are withdrawn after a slash has occurred.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSlashEvent {
    #[prost(uint64, tag = "1")]
    pub validator_period: u64,
    #[prost(string, tag = "2")]
    pub fraction: ::prost::alloc::string::String,
}
impl ::prost::Name for ValidatorSlashEvent {
    const NAME: &'static str = "ValidatorSlashEvent";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorSlashEvents is a collection of ValidatorSlashEvent messages.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSlashEvents {
    #[prost(message, repeated, tag = "1")]
    pub validator_slash_events: ::prost::alloc::vec::Vec<ValidatorSlashEvent>,
}
impl ::prost::Name for ValidatorSlashEvents {
    const NAME: &'static str = "ValidatorSlashEvents";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// FeePool is the global fee pool for distribution.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeePool {
    #[prost(message, repeated, tag = "1")]
    pub community_pool: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for FeePool {
    const NAME: &'static str = "FeePool";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// CommunityPoolSpendProposal details a proposal for use of community funds,
/// together with how many coins are proposed to be spent, and to which
/// recipient account.
///
/// Deprecated: Do not use. As of the Cosmos SDK release v0.47.x, there is no
/// longer a need for an explicit CommunityPoolSpendProposal. To spend community
/// pool funds, a simple MsgCommunityPoolSpend can be invoked from the x/gov
/// module via a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolSpendProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for CommunityPoolSpendProposal {
    const NAME: &'static str = "CommunityPoolSpendProposal";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// DelegatorStartingInfo represents the starting info for a delegator reward
/// period. It tracks the previous validator period, the delegation's amount of
/// staking token, and the creation height (to check later on if any slashes have
/// occurred). NOTE: Even though validators are slashed to whole staking tokens,
/// the delegators within the validator may be left with less than a full token,
/// thus sdk.Dec is used.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorStartingInfo {
    #[prost(uint64, tag = "1")]
    pub previous_period: u64,
    #[prost(string, tag = "2")]
    pub stake: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub height: u64,
}
impl ::prost::Name for DelegatorStartingInfo {
    const NAME: &'static str = "DelegatorStartingInfo";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// DelegationDelegatorReward represents the properties
/// of a delegator's delegation reward.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationDelegatorReward {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub reward: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for DelegationDelegatorReward {
    const NAME: &'static str = "DelegationDelegatorReward";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// CommunityPoolSpendProposalWithDeposit defines a CommunityPoolSpendProposal
/// with a deposit
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolSpendProposalWithDeposit {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub amount: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub deposit: ::prost::alloc::string::String,
}
impl ::prost::Name for CommunityPoolSpendProposalWithDeposit {
    const NAME: &'static str = "CommunityPoolSpendProposalWithDeposit";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// DelegatorWithdrawInfo is the address for where distributions rewards are
/// withdrawn to by default this struct is only used at genesis to feed in
/// default withdraw addresses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorWithdrawInfo {
    /// delegator_address is the address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// withdraw_address is the address to withdraw the delegation rewards to.
    #[prost(string, tag = "2")]
    pub withdraw_address: ::prost::alloc::string::String,
}
impl ::prost::Name for DelegatorWithdrawInfo {
    const NAME: &'static str = "DelegatorWithdrawInfo";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorOutstandingRewardsRecord is used for import/export via genesis json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorOutstandingRewardsRecord {
    /// validator_address is the address of the validator.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// outstanding_rewards represents the outstanding rewards of a validator.
    #[prost(message, repeated, tag = "2")]
    pub outstanding_rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for ValidatorOutstandingRewardsRecord {
    const NAME: &'static str = "ValidatorOutstandingRewardsRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorAccumulatedCommissionRecord is used for import / export via genesis
/// json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorAccumulatedCommissionRecord {
    /// validator_address is the address of the validator.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// accumulated is the accumulated commission of a validator.
    #[prost(message, optional, tag = "2")]
    pub accumulated: ::core::option::Option<ValidatorAccumulatedCommission>,
}
impl ::prost::Name for ValidatorAccumulatedCommissionRecord {
    const NAME: &'static str = "ValidatorAccumulatedCommissionRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorHistoricalRewardsRecord is used for import / export via genesis
/// json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorHistoricalRewardsRecord {
    /// validator_address is the address of the validator.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// period defines the period the historical rewards apply to.
    #[prost(uint64, tag = "2")]
    pub period: u64,
    /// rewards defines the historical rewards of a validator.
    #[prost(message, optional, tag = "3")]
    pub rewards: ::core::option::Option<ValidatorHistoricalRewards>,
}
impl ::prost::Name for ValidatorHistoricalRewardsRecord {
    const NAME: &'static str = "ValidatorHistoricalRewardsRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorCurrentRewardsRecord is used for import / export via genesis json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorCurrentRewardsRecord {
    /// validator_address is the address of the validator.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// rewards defines the current rewards of a validator.
    #[prost(message, optional, tag = "2")]
    pub rewards: ::core::option::Option<ValidatorCurrentRewards>,
}
impl ::prost::Name for ValidatorCurrentRewardsRecord {
    const NAME: &'static str = "ValidatorCurrentRewardsRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// DelegatorStartingInfoRecord used for import / export via genesis json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorStartingInfoRecord {
    /// delegator_address is the address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address is the address of the validator.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// starting_info defines the starting info of a delegator.
    #[prost(message, optional, tag = "3")]
    pub starting_info: ::core::option::Option<DelegatorStartingInfo>,
}
impl ::prost::Name for DelegatorStartingInfoRecord {
    const NAME: &'static str = "DelegatorStartingInfoRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorSlashEventRecord is used for import / export via genesis json.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSlashEventRecord {
    /// validator_address is the address of the validator.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// height defines the block height at which the slash event occurred.
    #[prost(uint64, tag = "2")]
    pub height: u64,
    /// period is the period of the slash event.
    #[prost(uint64, tag = "3")]
    pub period: u64,
    /// validator_slash_event describes the slash event.
    #[prost(message, optional, tag = "4")]
    pub validator_slash_event: ::core::option::Option<ValidatorSlashEvent>,
}
impl ::prost::Name for ValidatorSlashEventRecord {
    const NAME: &'static str = "ValidatorSlashEventRecord";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the distribution module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines all the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// fee_pool defines the fee pool at genesis.
    #[prost(message, optional, tag = "2")]
    pub fee_pool: ::core::option::Option<FeePool>,
    /// fee_pool defines the delegator withdraw infos at genesis.
    #[prost(message, repeated, tag = "3")]
    pub delegator_withdraw_infos: ::prost::alloc::vec::Vec<DelegatorWithdrawInfo>,
    /// fee_pool defines the previous proposer at genesis.
    #[prost(string, tag = "4")]
    pub previous_proposer: ::prost::alloc::string::String,
    /// fee_pool defines the outstanding rewards of all validators at genesis.
    #[prost(message, repeated, tag = "5")]
    pub outstanding_rewards: ::prost::alloc::vec::Vec<ValidatorOutstandingRewardsRecord>,
    /// fee_pool defines the accumulated commissions of all validators at genesis.
    #[prost(message, repeated, tag = "6")]
    pub validator_accumulated_commissions:
        ::prost::alloc::vec::Vec<ValidatorAccumulatedCommissionRecord>,
    /// fee_pool defines the historical rewards of all validators at genesis.
    #[prost(message, repeated, tag = "7")]
    pub validator_historical_rewards: ::prost::alloc::vec::Vec<ValidatorHistoricalRewardsRecord>,
    /// fee_pool defines the current rewards of all validators at genesis.
    #[prost(message, repeated, tag = "8")]
    pub validator_current_rewards: ::prost::alloc::vec::Vec<ValidatorCurrentRewardsRecord>,
    /// fee_pool defines the delegator starting infos at genesis.
    #[prost(message, repeated, tag = "9")]
    pub delegator_starting_infos: ::prost::alloc::vec::Vec<DelegatorStartingInfoRecord>,
    /// fee_pool defines the validator slash events at genesis.
    #[prost(message, repeated, tag = "10")]
    pub validator_slash_events: ::prost::alloc::vec::Vec<ValidatorSlashEventRecord>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorDistributionInfoRequest is the request type for the Query/ValidatorDistributionInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDistributionInfoRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorDistributionInfoRequest {
    const NAME: &'static str = "QueryValidatorDistributionInfoRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorDistributionInfoResponse is the response type for the Query/ValidatorDistributionInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDistributionInfoResponse {
    /// operator_address defines the validator operator address.
    #[prost(string, tag = "1")]
    pub operator_address: ::prost::alloc::string::String,
    /// self_bond_rewards defines the self delegations rewards.
    #[prost(message, repeated, tag = "2")]
    pub self_bond_rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    /// commission defines the commission the validator received.
    #[prost(message, repeated, tag = "3")]
    pub commission: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for QueryValidatorDistributionInfoResponse {
    const NAME: &'static str = "QueryValidatorDistributionInfoResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorOutstandingRewardsRequest is the request type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorOutstandingRewardsRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorOutstandingRewardsRequest {
    const NAME: &'static str = "QueryValidatorOutstandingRewardsRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorOutstandingRewardsResponse is the response type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorOutstandingRewardsResponse {
    #[prost(message, optional, tag = "1")]
    pub rewards: ::core::option::Option<ValidatorOutstandingRewards>,
}
impl ::prost::Name for QueryValidatorOutstandingRewardsResponse {
    const NAME: &'static str = "QueryValidatorOutstandingRewardsResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorCommissionRequest is the request type for the
/// Query/ValidatorCommission RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorCommissionRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorCommissionRequest {
    const NAME: &'static str = "QueryValidatorCommissionRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorCommissionResponse is the response type for the
/// Query/ValidatorCommission RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorCommissionResponse {
    /// commission defines the commission the validator received.
    #[prost(message, optional, tag = "1")]
    pub commission: ::core::option::Option<ValidatorAccumulatedCommission>,
}
impl ::prost::Name for QueryValidatorCommissionResponse {
    const NAME: &'static str = "QueryValidatorCommissionResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorSlashesRequest is the request type for the
/// Query/ValidatorSlashes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorSlashesRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// starting_height defines the optional starting height to query the slashes.
    #[prost(uint64, tag = "2")]
    pub starting_height: u64,
    /// starting_height defines the optional ending height to query the slashes.
    #[prost(uint64, tag = "3")]
    pub ending_height: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryValidatorSlashesRequest {
    const NAME: &'static str = "QueryValidatorSlashesRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorSlashesResponse is the response type for the
/// Query/ValidatorSlashes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorSlashesResponse {
    /// slashes defines the slashes the validator received.
    #[prost(message, repeated, tag = "1")]
    pub slashes: ::prost::alloc::vec::Vec<ValidatorSlashEvent>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryValidatorSlashesResponse {
    const NAME: &'static str = "QueryValidatorSlashesResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationRewardsRequest is the request type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegationRewardsRequest {
    const NAME: &'static str = "QueryDelegationRewardsRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationRewardsResponse is the response type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsResponse {
    /// rewards defines the rewards accrued by a delegation.
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for QueryDelegationRewardsResponse {
    const NAME: &'static str = "QueryDelegationRewardsResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationTotalRewardsRequest is the request type for the
/// Query/DelegationTotalRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationTotalRewardsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegationTotalRewardsRequest {
    const NAME: &'static str = "QueryDelegationTotalRewardsRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationTotalRewardsResponse is the response type for the
/// Query/DelegationTotalRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationTotalRewardsResponse {
    /// rewards defines all the rewards accrued by a delegator.
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<DelegationDelegatorReward>,
    /// total defines the sum of all the rewards.
    #[prost(message, repeated, tag = "2")]
    pub total: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for QueryDelegationTotalRewardsResponse {
    const NAME: &'static str = "QueryDelegationTotalRewardsResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorsRequest is the request type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegatorValidatorsRequest {
    const NAME: &'static str = "QueryDelegatorValidatorsRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorsResponse is the response type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsResponse {
    /// validators defines the validators a delegator is delegating for.
    #[prost(string, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for QueryDelegatorValidatorsResponse {
    const NAME: &'static str = "QueryDelegatorValidatorsResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryCommunityPoolRequest is the request type for the Query/CommunityPool RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCommunityPoolRequest {}
impl ::prost::Name for QueryCommunityPoolRequest {
    const NAME: &'static str = "QueryCommunityPoolRequest";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// QueryCommunityPoolResponse is the response type for the Query/CommunityPool
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCommunityPoolResponse {
    /// pool defines community pool's coins.
    #[prost(message, repeated, tag = "1")]
    pub pool: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
impl ::prost::Name for QueryCommunityPoolResponse {
    const NAME: &'static str = "QueryCommunityPoolResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetWithdrawAddress sets the withdraw address for
/// a delegator (or validator self-delegation).
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
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetWithdrawAddressResponse defines the Msg/SetWithdrawAddress response
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetWithdrawAddressResponse {}
impl ::prost::Name for MsgSetWithdrawAddressResponse {
    const NAME: &'static str = "MsgSetWithdrawAddressResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgWithdrawDelegatorReward represents delegation withdrawal to a delegator
/// from a single validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawDelegatorReward {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgWithdrawDelegatorReward {
    const NAME: &'static str = "MsgWithdrawDelegatorReward";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgWithdrawDelegatorRewardResponse defines the Msg/WithdrawDelegatorReward
/// response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawDelegatorRewardResponse {
    /// Since: cosmos-sdk 0.46
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgWithdrawDelegatorRewardResponse {
    const NAME: &'static str = "MsgWithdrawDelegatorRewardResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgWithdrawValidatorCommission withdraws the full commission to the validator
/// address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawValidatorCommission {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgWithdrawValidatorCommission {
    const NAME: &'static str = "MsgWithdrawValidatorCommission";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgWithdrawValidatorCommissionResponse defines the
/// Msg/WithdrawValidatorCommission response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawValidatorCommissionResponse {
    /// Since: cosmos-sdk 0.46
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgWithdrawValidatorCommissionResponse {
    const NAME: &'static str = "MsgWithdrawValidatorCommissionResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgFundCommunityPool allows an account to directly
/// fund the community pool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgFundCommunityPool {
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgFundCommunityPool {
    const NAME: &'static str = "MsgFundCommunityPool";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgFundCommunityPoolResponse defines the Msg/FundCommunityPool response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgFundCommunityPoolResponse {}
impl ::prost::Name for MsgFundCommunityPoolResponse {
    const NAME: &'static str = "MsgFundCommunityPoolResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
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
    /// params defines the x/distribution parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgCommunityPoolSpend defines a message for sending tokens from the community
/// pool to another account. This message is typically executed via a governance
/// proposal with the governance module being the executing authority.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCommunityPoolSpend {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgCommunityPoolSpend {
    const NAME: &'static str = "MsgCommunityPoolSpend";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgCommunityPoolSpendResponse defines the response to executing a
/// MsgCommunityPoolSpend message.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCommunityPoolSpendResponse {}
impl ::prost::Name for MsgCommunityPoolSpendResponse {
    const NAME: &'static str = "MsgCommunityPoolSpendResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// DepositValidatorRewardsPool defines the request structure to provide
/// additional rewards to delegators from a specific validator.
///
/// Since: cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositValidatorRewardsPool {
    #[prost(string, tag = "1")]
    pub depositor: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgDepositValidatorRewardsPool {
    const NAME: &'static str = "MsgDepositValidatorRewardsPool";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
/// MsgDepositValidatorRewardsPoolResponse defines the response to executing a
/// MsgDepositValidatorRewardsPool message.
///
/// Since: cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositValidatorRewardsPoolResponse {}
impl ::prost::Name for MsgDepositValidatorRewardsPoolResponse {
    const NAME: &'static str = "MsgDepositValidatorRewardsPoolResponse";
    const PACKAGE: &'static str = "cosmos.distribution.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.distribution.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.distribution.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
