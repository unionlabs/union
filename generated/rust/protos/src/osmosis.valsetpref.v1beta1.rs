// @generated
/// ValidatorPreference defines the message structure for
/// CreateValidatorSetPreference. It allows a user to set {val_addr, weight} in
/// state. If a user does not have a validator set preference list set, and has
/// staked, make their preference list default to their current staking
/// distribution.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorPreference {
    /// val_oper_address holds the validator address the user wants to delegate
    /// funds to.
    #[prost(string, tag = "1")]
    pub val_oper_address: ::prost::alloc::string::String,
    /// weight is decimal between 0 and 1, and they all sum to 1.
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}
impl ::prost::Name for ValidatorPreference {
    const NAME: &'static str = "ValidatorPreference";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorSetPreferences defines a delegator's validator set preference.
/// It contains a list of (validator, percent_allocation) pairs.
/// The percent allocation are arranged in decimal notation from 0 to 1 and must
/// add up to 1.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetPreferences {
    /// preference holds {valAddr, weight} for the user who created it.
    #[prost(message, repeated, tag = "2")]
    pub preferences: ::prost::alloc::vec::Vec<ValidatorPreference>,
}
impl ::prost::Name for ValidatorSetPreferences {
    const NAME: &'static str = "ValidatorSetPreferences";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// Request type for UserValidatorPreferences.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserValidatorPreferencesRequest {
    /// user account address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for UserValidatorPreferencesRequest {
    const NAME: &'static str = "UserValidatorPreferencesRequest";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// Response type the QueryUserValidatorPreferences query request
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserValidatorPreferencesResponse {
    #[prost(message, repeated, tag = "1")]
    pub preferences: ::prost::alloc::vec::Vec<ValidatorPreference>,
}
impl ::prost::Name for UserValidatorPreferencesResponse {
    const NAME: &'static str = "UserValidatorPreferencesResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateValidatorSetPreference is a list that holds validator-set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetValidatorSetPreference {
    /// delegator is the user who is trying to create a validator-set.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// list of {valAddr, weight} to delegate to
    #[prost(message, repeated, tag = "2")]
    pub preferences: ::prost::alloc::vec::Vec<ValidatorPreference>,
}
impl ::prost::Name for MsgSetValidatorSetPreference {
    const NAME: &'static str = "MsgSetValidatorSetPreference";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetValidatorSetPreferenceResponse {}
impl ::prost::Name for MsgSetValidatorSetPreferenceResponse {
    const NAME: &'static str = "MsgSetValidatorSetPreferenceResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// MsgDelegateToValidatorSet allows users to delegate to an existing
/// validator-set
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateToValidatorSet {
    /// delegator is the user who is trying to delegate.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// the amount of tokens the user is trying to delegate.
    /// For ex: delegate 10osmo with validator-set {ValA -> 0.5, ValB -> 0.3, ValC
    /// -> 0.2} our staking logic would attempt to delegate 5osmo to A , 3osmo to
    /// B, 2osmo to C.
    #[prost(message, optional, tag = "2")]
    pub coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgDelegateToValidatorSet {
    const NAME: &'static str = "MsgDelegateToValidatorSet";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateToValidatorSetResponse {}
impl ::prost::Name for MsgDelegateToValidatorSetResponse {
    const NAME: &'static str = "MsgDelegateToValidatorSetResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateFromValidatorSet {
    /// delegator is the user who is trying to undelegate.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// the amount the user wants to undelegate
    /// For ex: Undelegate 10osmo with validator-set {ValA -> 0.5, ValB -> 0.3,
    /// ValC
    /// -> 0.2} our undelegate logic would attempt to undelegate 5osmo from A ,
    /// 3osmo from B, 2osmo from C
    #[prost(message, optional, tag = "3")]
    pub coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUndelegateFromValidatorSet {
    const NAME: &'static str = "MsgUndelegateFromValidatorSet";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateFromValidatorSetResponse {}
impl ::prost::Name for MsgUndelegateFromValidatorSetResponse {
    const NAME: &'static str = "MsgUndelegateFromValidatorSetResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateFromRebalancedValidatorSet {
    /// delegator is the user who is trying to undelegate.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// the amount the user wants to undelegate
    /// For ex: Undelegate 50 osmo with validator-set {ValA -> 0.5, ValB -> 0.5}
    /// Our undelegate logic would first check the current delegation balance.
    /// If the user has 90 osmo delegated to ValA and 10 osmo delegated to ValB,
    /// the rebalanced validator set would be {ValA -> 0.9, ValB -> 0.1}
    /// So now the 45 osmo would be undelegated from ValA and 5 osmo would be
    /// undelegated from ValB.
    #[prost(message, optional, tag = "2")]
    pub coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUndelegateFromRebalancedValidatorSet {
    const NAME: &'static str = "MsgUndelegateFromRebalancedValidatorSet";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateFromRebalancedValidatorSetResponse {}
impl ::prost::Name for MsgUndelegateFromRebalancedValidatorSetResponse {
    const NAME: &'static str = "MsgUndelegateFromRebalancedValidatorSetResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRedelegateValidatorSet {
    /// delegator is the user who is trying to create a validator-set.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// list of {valAddr, weight} to delegate to
    #[prost(message, repeated, tag = "2")]
    pub preferences: ::prost::alloc::vec::Vec<ValidatorPreference>,
}
impl ::prost::Name for MsgRedelegateValidatorSet {
    const NAME: &'static str = "MsgRedelegateValidatorSet";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRedelegateValidatorSetResponse {}
impl ::prost::Name for MsgRedelegateValidatorSetResponse {
    const NAME: &'static str = "MsgRedelegateValidatorSetResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// MsgWithdrawDelegationRewards allows user to claim staking rewards from the
/// validator set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawDelegationRewards {
    /// delegator is the user who is trying to claim staking rewards.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgWithdrawDelegationRewards {
    const NAME: &'static str = "MsgWithdrawDelegationRewards";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawDelegationRewardsResponse {}
impl ::prost::Name for MsgWithdrawDelegationRewardsResponse {
    const NAME: &'static str = "MsgWithdrawDelegationRewardsResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
/// MsgDelegateBondedTokens breaks bonded lockup (by ID) of osmo, of
/// length <= 2 weeks and takes all that osmo and delegates according to
/// delegator's current validator set preference.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateBondedTokens {
    /// delegator is the user who is trying to force unbond osmo and delegate.
    #[prost(string, tag = "1")]
    pub delegator: ::prost::alloc::string::String,
    /// lockup id of osmo in the pool
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
}
impl ::prost::Name for MsgDelegateBondedTokens {
    const NAME: &'static str = "MsgDelegateBondedTokens";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateBondedTokensResponse {}
impl ::prost::Name for MsgDelegateBondedTokensResponse {
    const NAME: &'static str = "MsgDelegateBondedTokensResponse";
    const PACKAGE: &'static str = "osmosis.valsetpref.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.valsetpref.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.valsetpref.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
