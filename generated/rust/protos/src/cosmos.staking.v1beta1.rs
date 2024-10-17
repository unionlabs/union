// @generated
/// StakeAuthorization defines authorization for delegate/undelegate/redelegate.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StakeAuthorization {
    /// max_tokens specifies the maximum amount of tokens can be delegate to a validator. If it is
    /// empty, there is no spend limit and any amount of coins can be delegated.
    #[prost(message, optional, tag = "1")]
    pub max_tokens: ::core::option::Option<super::super::base::v1beta1::Coin>,
    /// authorization_type defines one of AuthorizationType.
    #[prost(enumeration = "AuthorizationType", tag = "4")]
    pub authorization_type: i32,
    /// validators is the oneof that represents either allow_list or deny_list
    #[prost(oneof = "stake_authorization::Validators", tags = "2, 3")]
    pub validators: ::core::option::Option<stake_authorization::Validators>,
}
/// Nested message and enum types in `StakeAuthorization`.
pub mod stake_authorization {
    /// Validators defines list of validator addresses.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ValidatorsList {
        #[prost(string, repeated, tag = "1")]
        pub address: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }
    impl ::prost::Name for ValidatorsList {
        const NAME: &'static str = "Validators";
        const PACKAGE: &'static str = "cosmos.staking.v1beta1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.staking.v1beta1.StakeAuthorization.{}", Self::NAME)
        }
    }
    /// validators is the oneof that represents either allow_list or deny_list
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Validators {
        /// allow_list specifies list of validator addresses to whom grantee can delegate tokens on behalf of granter's
        /// account.
        #[prost(message, tag = "2")]
        AllowList(ValidatorsList),
        /// deny_list specifies list of validator addresses to whom grantee can not delegate tokens.
        #[prost(message, tag = "3")]
        DenyList(ValidatorsList),
    }
}
impl ::prost::Name for StakeAuthorization {
    const NAME: &'static str = "StakeAuthorization";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// AuthorizationType defines the type of staking module authorization type
///
/// Since: cosmos-sdk 0.43
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AuthorizationType {
    /// AUTHORIZATION_TYPE_UNSPECIFIED specifies an unknown authorization type
    Unspecified = 0,
    /// AUTHORIZATION_TYPE_DELEGATE defines an authorization type for Msg/Delegate
    Delegate = 1,
    /// AUTHORIZATION_TYPE_UNDELEGATE defines an authorization type for Msg/Undelegate
    Undelegate = 2,
    /// AUTHORIZATION_TYPE_REDELEGATE defines an authorization type for Msg/BeginRedelegate
    Redelegate = 3,
    /// AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION defines an authorization type for Msg/MsgCancelUnbondingDelegation
    CancelUnbondingDelegation = 4,
}
impl AuthorizationType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            AuthorizationType::Unspecified => "AUTHORIZATION_TYPE_UNSPECIFIED",
            AuthorizationType::Delegate => "AUTHORIZATION_TYPE_DELEGATE",
            AuthorizationType::Undelegate => "AUTHORIZATION_TYPE_UNDELEGATE",
            AuthorizationType::Redelegate => "AUTHORIZATION_TYPE_REDELEGATE",
            AuthorizationType::CancelUnbondingDelegation => {
                "AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION"
            }
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AUTHORIZATION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "AUTHORIZATION_TYPE_DELEGATE" => Some(Self::Delegate),
            "AUTHORIZATION_TYPE_UNDELEGATE" => Some(Self::Undelegate),
            "AUTHORIZATION_TYPE_REDELEGATE" => Some(Self::Redelegate),
            "AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION" => {
                Some(Self::CancelUnbondingDelegation)
            }
            _ => None,
        }
    }
}
/// HistoricalInfo contains header and validator information for a given block.
/// It is stored as part of staking module's state, which persists the `n` most
/// recent HistoricalInfo
/// (`n` is set by the staking module's `historical_entries` parameter).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HistoricalInfo {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::super::super::cometbft::types::v1::Header>,
    #[prost(message, repeated, tag = "2")]
    pub valset: ::prost::alloc::vec::Vec<Validator>,
}
impl ::prost::Name for HistoricalInfo {
    const NAME: &'static str = "HistoricalInfo";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// CommissionRates defines the initial commission rates to be used for creating
/// a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    #[prost(string, tag = "1")]
    pub rate: ::prost::alloc::string::String,
    /// max_rate defines the maximum commission rate which validator can ever charge, as a fraction.
    #[prost(string, tag = "2")]
    pub max_rate: ::prost::alloc::string::String,
    /// max_change_rate defines the maximum daily increase of the validator commission, as a fraction.
    #[prost(string, tag = "3")]
    pub max_change_rate: ::prost::alloc::string::String,
}
impl ::prost::Name for CommissionRates {
    const NAME: &'static str = "CommissionRates";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Commission defines commission parameters for a given validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commission {
    /// commission_rates defines the initial commission rates to be used for creating a validator.
    #[prost(message, optional, tag = "1")]
    pub commission_rates: ::core::option::Option<CommissionRates>,
    /// update_time is the last time the commission rate was changed.
    #[prost(message, optional, tag = "2")]
    pub update_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for Commission {
    const NAME: &'static str = "Commission";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Description defines a validator description.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Description {
    /// moniker defines a human-readable name for the validator.
    #[prost(string, tag = "1")]
    pub moniker: ::prost::alloc::string::String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string, tag = "2")]
    pub identity: ::prost::alloc::string::String,
    /// website defines an optional website link.
    #[prost(string, tag = "3")]
    pub website: ::prost::alloc::string::String,
    /// security_contact defines an optional email for security contact.
    #[prost(string, tag = "4")]
    pub security_contact: ::prost::alloc::string::String,
    /// details define other optional details.
    #[prost(string, tag = "5")]
    pub details: ::prost::alloc::string::String,
}
impl ::prost::Name for Description {
    const NAME: &'static str = "Description";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Validator defines a validator, together with the total amount of the
/// Validator's bond shares and their exchange rate to coins. Slashing results in
/// a decrease in the exchange rate, allowing correct calculation of future
/// undelegations without iterating over delegators. When coins are delegated to
/// this validator, the validator is credited with a delegation whose number of
/// bond shares is based on the amount of coins delegated divided by the current
/// exchange rate. Voting power can be calculated as total bonded shares
/// multiplied by exchange rate.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// operator_address defines the address of the validator's operator; bech encoded in JSON.
    #[prost(string, tag = "1")]
    pub operator_address: ::prost::alloc::string::String,
    /// consensus_pubkey is the consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "2")]
    pub consensus_pubkey: ::core::option::Option<::pbjson_types::Any>,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    #[prost(bool, tag = "3")]
    pub jailed: bool,
    /// status is the validator status (bonded/unbonding/unbonded).
    #[prost(enumeration = "BondStatus", tag = "4")]
    pub status: i32,
    /// tokens define the delegated tokens (incl. self-delegation).
    #[prost(string, tag = "5")]
    pub tokens: ::prost::alloc::string::String,
    /// delegator_shares defines total shares issued to a validator's delegators.
    #[prost(string, tag = "6")]
    pub delegator_shares: ::prost::alloc::string::String,
    /// description defines the description terms for the validator.
    #[prost(message, optional, tag = "7")]
    pub description: ::core::option::Option<Description>,
    /// unbonding_height defines, if unbonding, the height at which this validator has begun unbonding.
    #[prost(int64, tag = "8")]
    pub unbonding_height: i64,
    /// unbonding_time defines, if unbonding, the min time for the validator to complete unbonding.
    #[prost(message, optional, tag = "9")]
    pub unbonding_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// commission defines the commission parameters.
    #[prost(message, optional, tag = "10")]
    pub commission: ::core::option::Option<Commission>,
    /// min_self_delegation is the validator's self declared minimum self delegation.
    #[prost(string, tag = "11")]
    pub min_self_delegation: ::prost::alloc::string::String,
    /// strictly positive if this validator's unbonding has been stopped by external modules
    #[prost(int64, tag = "12")]
    pub unbonding_on_hold_ref_count: i64,
    /// list of unbonding ids, each uniquely identifying an unbonding of this validator
    #[prost(uint64, repeated, tag = "13")]
    pub unbonding_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// ValAddresses defines a repeated set of validator addresses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValAddresses {
    #[prost(string, repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ValAddresses {
    const NAME: &'static str = "ValAddresses";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// DVPair is struct that just has a delegator-validator pair with no other data.
/// It is intended to be used as a marshalable pointer. For example, a DVPair can
/// be used to construct the key to getting an UnbondingDelegation from state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvPair {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for DvPair {
    const NAME: &'static str = "DVPair";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// DVPairs defines an array of DVPair objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvPairs {
    #[prost(message, repeated, tag = "1")]
    pub pairs: ::prost::alloc::vec::Vec<DvPair>,
}
impl ::prost::Name for DvPairs {
    const NAME: &'static str = "DVPairs";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// DVVTriplet is struct that just has a delegator-validator-validator triplet
/// with no other data. It is intended to be used as a marshalable pointer. For
/// example, a DVVTriplet can be used to construct the key to getting a
/// Redelegation from state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvvTriplet {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
}
impl ::prost::Name for DvvTriplet {
    const NAME: &'static str = "DVVTriplet";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// DVVTriplets defines an array of DVVTriplet objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvvTriplets {
    #[prost(message, repeated, tag = "1")]
    pub triplets: ::prost::alloc::vec::Vec<DvvTriplet>,
}
impl ::prost::Name for DvvTriplets {
    const NAME: &'static str = "DVVTriplets";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delegation {
    /// delegator_address is the encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address is the encoded address of the validator.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// shares define the delegation shares received.
    #[prost(string, tag = "3")]
    pub shares: ::prost::alloc::string::String,
}
impl ::prost::Name for Delegation {
    const NAME: &'static str = "Delegation";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// UnbondingDelegation stores all of a single delegator's unbonding bonds
/// for a single validator in an time-ordered list.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnbondingDelegation {
    /// delegator_address is the encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address is the encoded address of the validator.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// entries are the unbonding delegation entries.
    ///
    /// unbonding delegation entries
    #[prost(message, repeated, tag = "3")]
    pub entries: ::prost::alloc::vec::Vec<UnbondingDelegationEntry>,
}
impl ::prost::Name for UnbondingDelegation {
    const NAME: &'static str = "UnbondingDelegation";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// UnbondingDelegationEntry defines an unbonding object with relevant metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnbondingDelegationEntry {
    /// creation_height is the height which the unbonding took place.
    #[prost(int64, tag = "1")]
    pub creation_height: i64,
    /// completion_time is the unix time for unbonding completion.
    #[prost(message, optional, tag = "2")]
    pub completion_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// initial_balance defines the tokens initially scheduled to receive at completion.
    #[prost(string, tag = "3")]
    pub initial_balance: ::prost::alloc::string::String,
    /// balance defines the tokens to receive at completion.
    #[prost(string, tag = "4")]
    pub balance: ::prost::alloc::string::String,
    /// Incrementing id that uniquely identifies this entry
    #[prost(uint64, tag = "5")]
    pub unbonding_id: u64,
    /// Strictly positive if this entry's unbonding has been stopped by external modules
    #[prost(int64, tag = "6")]
    pub unbonding_on_hold_ref_count: i64,
}
impl ::prost::Name for UnbondingDelegationEntry {
    const NAME: &'static str = "UnbondingDelegationEntry";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// RedelegationEntry defines a redelegation object with relevant metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationEntry {
    /// creation_height  defines the height which the redelegation took place.
    #[prost(int64, tag = "1")]
    pub creation_height: i64,
    /// completion_time defines the unix time for redelegation completion.
    #[prost(message, optional, tag = "2")]
    pub completion_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// initial_balance defines the initial balance when redelegation started.
    #[prost(string, tag = "3")]
    pub initial_balance: ::prost::alloc::string::String,
    /// shares_dst is the amount of destination-validator shares created by redelegation.
    #[prost(string, tag = "4")]
    pub shares_dst: ::prost::alloc::string::String,
    /// Incrementing id that uniquely identifies this entry
    #[prost(uint64, tag = "5")]
    pub unbonding_id: u64,
    /// Strictly positive if this entry's unbonding has been stopped by external modules
    #[prost(int64, tag = "6")]
    pub unbonding_on_hold_ref_count: i64,
}
impl ::prost::Name for RedelegationEntry {
    const NAME: &'static str = "RedelegationEntry";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Redelegation contains the list of a particular delegator's redelegating bonds
/// from a particular source validator to a particular destination validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Redelegation {
    /// delegator_address is the bech32-encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_src_address is the validator redelegation source operator address.
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    /// validator_dst_address is the validator redelegation destination operator address.
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
    /// entries are the redelegation entries.
    ///
    /// redelegation entries
    #[prost(message, repeated, tag = "4")]
    pub entries: ::prost::alloc::vec::Vec<RedelegationEntry>,
}
impl ::prost::Name for Redelegation {
    const NAME: &'static str = "Redelegation";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the x/staking module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// unbonding_time is the time duration of unbonding.
    #[prost(message, optional, tag = "1")]
    pub unbonding_time: ::core::option::Option<::pbjson_types::Duration>,
    /// max_validators is the maximum number of validators.
    #[prost(uint32, tag = "2")]
    pub max_validators: u32,
    /// max_entries is the max entries for either unbonding delegation or redelegation (per pair/trio).
    #[prost(uint32, tag = "3")]
    pub max_entries: u32,
    /// historical_entries is the number of historical entries to persist.
    #[deprecated]
    #[prost(uint32, tag = "4")]
    pub historical_entries: u32,
    /// bond_denom defines the bondable coin denomination.
    #[prost(string, tag = "5")]
    pub bond_denom: ::prost::alloc::string::String,
    /// min_commission_rate is the chain-wide minimum commission rate that a validator can charge their delegators
    #[prost(string, tag = "6")]
    pub min_commission_rate: ::prost::alloc::string::String,
    /// key_rotation_fee is fee to be spent when rotating validator's key
    /// (either consensus pubkey or operator key)
    #[prost(message, optional, tag = "7")]
    pub key_rotation_fee: ::core::option::Option<super::super::base::v1beta1::Coin>,
    /// The percentage of validators that can be jailed before forcing a validator set rotation
    #[prost(uint32, tag = "8")]
    pub jailed_validator_threshold: u32,
    /// The number of blocks between regular validator set rotations (between epochs)
    #[prost(int64, tag = "9")]
    pub epoch_length: i64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// DelegationResponse is equivalent to Delegation except that it contains a
/// balance in addition to shares which is more suitable for client responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationResponse {
    #[prost(message, optional, tag = "1")]
    pub delegation: ::core::option::Option<Delegation>,
    #[prost(message, optional, tag = "2")]
    pub balance: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for DelegationResponse {
    const NAME: &'static str = "DelegationResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// RedelegationEntryResponse is equivalent to a RedelegationEntry except that it
/// contains a balance in addition to shares which is more suitable for client
/// responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationEntryResponse {
    #[prost(message, optional, tag = "1")]
    pub redelegation_entry: ::core::option::Option<RedelegationEntry>,
    #[prost(string, tag = "4")]
    pub balance: ::prost::alloc::string::String,
}
impl ::prost::Name for RedelegationEntryResponse {
    const NAME: &'static str = "RedelegationEntryResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// RedelegationResponse is equivalent to a Redelegation except that its entries
/// contain a balance in addition to shares which is more suitable for client
/// responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationResponse {
    #[prost(message, optional, tag = "1")]
    pub redelegation: ::core::option::Option<Redelegation>,
    #[prost(message, repeated, tag = "2")]
    pub entries: ::prost::alloc::vec::Vec<RedelegationEntryResponse>,
}
impl ::prost::Name for RedelegationResponse {
    const NAME: &'static str = "RedelegationResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// Pool is used for tracking bonded and not-bonded token supply of the bond
/// denomination.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pool {
    #[prost(string, tag = "1")]
    pub not_bonded_tokens: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub bonded_tokens: ::prost::alloc::string::String,
}
impl ::prost::Name for Pool {
    const NAME: &'static str = "Pool";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorUpdates defines an array of abci.ValidatorUpdate objects.
/// TODO: explore moving this to proto/cosmos/base to separate modules from tendermint dependence
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorUpdates {
    #[prost(message, repeated, tag = "1")]
    pub updates: ::prost::alloc::vec::Vec<super::super::super::cometbft::abci::v1::ValidatorUpdate>,
}
impl ::prost::Name for ValidatorUpdates {
    const NAME: &'static str = "ValidatorUpdates";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// ConsPubKeyRotationHistory contains a validator's consensus public key rotation history.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsPubKeyRotationHistory {
    /// operator_address defines the address of the validator's operator; bech encoded in JSON.
    #[prost(bytes = "vec", tag = "1")]
    pub operator_address: ::prost::alloc::vec::Vec<u8>,
    /// old_cons_pubkey is the old consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "2")]
    pub old_cons_pubkey: ::core::option::Option<::pbjson_types::Any>,
    /// new_cons_pubkey is the new consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "3")]
    pub new_cons_pubkey: ::core::option::Option<::pbjson_types::Any>,
    /// height defines the block height at which the rotation event occurred.
    #[prost(uint64, tag = "4")]
    pub height: u64,
    /// fee holds the amount of fee deduced for the rotation.
    #[prost(message, optional, tag = "5")]
    pub fee: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for ConsPubKeyRotationHistory {
    const NAME: &'static str = "ConsPubKeyRotationHistory";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// ValAddrsOfRotatedConsKeys contains the array of validator addresses which rotated their keys
/// This is to block the validator's next rotation till unbonding period.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValAddrsOfRotatedConsKeys {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for ValAddrsOfRotatedConsKeys {
    const NAME: &'static str = "ValAddrsOfRotatedConsKeys";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// BondStatus is the status of a validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BondStatus {
    /// UNSPECIFIED defines an invalid validator status.
    Unspecified = 0,
    /// UNBONDED defines a validator that is not bonded.
    Unbonded = 1,
    /// UNBONDING defines a validator that is unbonding.
    Unbonding = 2,
    /// BONDED defines a validator that is bonded.
    Bonded = 3,
}
impl BondStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BondStatus::Unspecified => "BOND_STATUS_UNSPECIFIED",
            BondStatus::Unbonded => "BOND_STATUS_UNBONDED",
            BondStatus::Unbonding => "BOND_STATUS_UNBONDING",
            BondStatus::Bonded => "BOND_STATUS_BONDED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BOND_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "BOND_STATUS_UNBONDED" => Some(Self::Unbonded),
            "BOND_STATUS_UNBONDING" => Some(Self::Unbonding),
            "BOND_STATUS_BONDED" => Some(Self::Bonded),
            _ => None,
        }
    }
}
/// Infraction indicates the infraction a validator committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Infraction {
    /// UNSPECIFIED defines an empty infraction.
    Unspecified = 0,
    /// DOUBLE_SIGN defines a validator that double-signs a block.
    DoubleSign = 1,
    /// DOWNTIME defines a validator that missed signing too many blocks.
    Downtime = 2,
}
impl Infraction {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Infraction::Unspecified => "INFRACTION_UNSPECIFIED",
            Infraction::DoubleSign => "INFRACTION_DOUBLE_SIGN",
            Infraction::Downtime => "INFRACTION_DOWNTIME",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "INFRACTION_UNSPECIFIED" => Some(Self::Unspecified),
            "INFRACTION_DOUBLE_SIGN" => Some(Self::DoubleSign),
            "INFRACTION_DOWNTIME" => Some(Self::Downtime),
            _ => None,
        }
    }
}
/// GenesisState defines the staking module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines all the parameters of related to deposit.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// last_total_power tracks the total amounts of bonded tokens recorded during
    /// the previous end block.
    #[prost(bytes = "vec", tag = "2")]
    pub last_total_power: ::prost::alloc::vec::Vec<u8>,
    /// last_validator_powers is a special index that provides a historical list
    /// of the last-block's bonded validators.
    #[prost(message, repeated, tag = "3")]
    pub last_validator_powers: ::prost::alloc::vec::Vec<LastValidatorPower>,
    /// validators defines the validator set at genesis.
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// delegations defines the delegations active at genesis.
    #[prost(message, repeated, tag = "5")]
    pub delegations: ::prost::alloc::vec::Vec<Delegation>,
    /// unbonding_delegations defines the unbonding delegations active at genesis.
    #[prost(message, repeated, tag = "6")]
    pub unbonding_delegations: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// redelegations defines the redelegations active at genesis.
    #[prost(message, repeated, tag = "7")]
    pub redelegations: ::prost::alloc::vec::Vec<Redelegation>,
    /// exported defines a bool to identify whether the chain dealing with exported or initialized genesis.
    #[prost(bool, tag = "8")]
    pub exported: bool,
    #[prost(message, repeated, tag = "9")]
    pub rotation_index_records: ::prost::alloc::vec::Vec<RotationIndexRecord>,
    #[prost(message, repeated, tag = "10")]
    pub rotation_history: ::prost::alloc::vec::Vec<ConsPubKeyRotationHistory>,
    #[prost(message, repeated, tag = "11")]
    pub rotation_queue: ::prost::alloc::vec::Vec<RotationQueueRecord>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// LastValidatorPower required for validator set update logic.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastValidatorPower {
    /// address is the address of the validator.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// power defines the power of the validator.
    #[prost(int64, tag = "2")]
    pub power: i64,
}
impl ::prost::Name for LastValidatorPower {
    const NAME: &'static str = "LastValidatorPower";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// contains address as bytes and time as int64
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RotationIndexRecord {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for RotationIndexRecord {
    const NAME: &'static str = "RotationIndexRecord";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RotationQueueRecord {
    #[prost(message, optional, tag = "1")]
    pub val_addrs: ::core::option::Option<ValAddrsOfRotatedConsKeys>,
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for RotationQueueRecord {
    const NAME: &'static str = "RotationQueueRecord";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorsRequest is request type for Query/Validators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorsRequest {
    /// status enables to query for validators matching a given status.
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryValidatorsRequest {
    const NAME: &'static str = "QueryValidatorsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorInfo contains the validator's address and public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfo {
    /// consensus_address is the consensus address of the validator.
    #[prost(string, tag = "1")]
    pub consensus_address: ::prost::alloc::string::String,
}
impl ::prost::Name for ValidatorInfo {
    const NAME: &'static str = "ValidatorInfo";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorsResponse is response type for the Query/Validators RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorsResponse {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// validator_info contains additional information for each validator.
    /// The order of the elements in this list corresponds to the order of the elements in the validators list.
    /// For example, if you want the ValidatorInfo for the third validator in the validators list,
    /// you should look at the third element in the validator_info list.
    #[prost(message, repeated, tag = "2")]
    pub validator_info: ::prost::alloc::vec::Vec<ValidatorInfo>,
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryValidatorsResponse {
    const NAME: &'static str = "QueryValidatorsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorRequest is response type for the Query/Validator RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorRequest {
    const NAME: &'static str = "QueryValidatorRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorResponse is response type for the Query/Validator RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorResponse {
    /// validator defines the validator info.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
}
impl ::prost::Name for QueryValidatorResponse {
    const NAME: &'static str = "QueryValidatorResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorDelegationsRequest is request type for the
/// Query/ValidatorDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDelegationsRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryValidatorDelegationsRequest {
    const NAME: &'static str = "QueryValidatorDelegationsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorDelegationsResponse is response type for the
/// Query/ValidatorDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub delegation_responses: ::prost::alloc::vec::Vec<DelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryValidatorDelegationsResponse {
    const NAME: &'static str = "QueryValidatorDelegationsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorUnbondingDelegationsRequest is required type for the
/// Query/ValidatorUnbondingDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorUnbondingDelegationsRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryValidatorUnbondingDelegationsRequest {
    const NAME: &'static str = "QueryValidatorUnbondingDelegationsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryValidatorUnbondingDelegationsResponse is response type for the
/// Query/ValidatorUnbondingDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorUnbondingDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub unbonding_responses: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryValidatorUnbondingDelegationsResponse {
    const NAME: &'static str = "QueryValidatorUnbondingDelegationsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationRequest is request type for the Query/Delegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegationRequest {
    const NAME: &'static str = "QueryDelegationRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationResponse is response type for the Query/Delegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationResponse {
    /// delegation_responses defines the delegation info of a delegation.
    #[prost(message, optional, tag = "1")]
    pub delegation_response: ::core::option::Option<DelegationResponse>,
}
impl ::prost::Name for QueryDelegationResponse {
    const NAME: &'static str = "QueryDelegationResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryUnbondingDelegationRequest is request type for the
/// Query/UnbondingDelegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnbondingDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryUnbondingDelegationRequest {
    const NAME: &'static str = "QueryUnbondingDelegationRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegationResponse is response type for the Query/UnbondingDelegation
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnbondingDelegationResponse {
    /// unbond defines the unbonding information of a delegation.
    #[prost(message, optional, tag = "1")]
    pub unbond: ::core::option::Option<UnbondingDelegation>,
}
impl ::prost::Name for QueryUnbondingDelegationResponse {
    const NAME: &'static str = "QueryUnbondingDelegationResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorDelegationsRequest is request type for the
/// Query/DelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorDelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDelegatorDelegationsRequest {
    const NAME: &'static str = "QueryDelegatorDelegationsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorDelegationsResponse is response type for the
/// Query/DelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorDelegationsResponse {
    /// delegation_responses defines all the delegations' info of a delegator.
    #[prost(message, repeated, tag = "1")]
    pub delegation_responses: ::prost::alloc::vec::Vec<DelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDelegatorDelegationsResponse {
    const NAME: &'static str = "QueryDelegatorDelegationsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorUnbondingDelegationsRequest is request type for the
/// Query/DelegatorUnbondingDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorUnbondingDelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDelegatorUnbondingDelegationsRequest {
    const NAME: &'static str = "QueryDelegatorUnbondingDelegationsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryUnbondingDelegatorDelegationsResponse is response type for the
/// Query/UnbondingDelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorUnbondingDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub unbonding_responses: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDelegatorUnbondingDelegationsResponse {
    const NAME: &'static str = "QueryDelegatorUnbondingDelegationsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryRedelegationsRequest is request type for the Query/Redelegations RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRedelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// src_validator_addr defines the validator address to redelegate from.
    #[prost(string, tag = "2")]
    pub src_validator_addr: ::prost::alloc::string::String,
    /// dst_validator_addr defines the validator address to redelegate to.
    #[prost(string, tag = "3")]
    pub dst_validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryRedelegationsRequest {
    const NAME: &'static str = "QueryRedelegationsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryRedelegationsResponse is response type for the Query/Redelegations RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRedelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub redelegation_responses: ::prost::alloc::vec::Vec<RedelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryRedelegationsResponse {
    const NAME: &'static str = "QueryRedelegationsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorsRequest is request type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDelegatorValidatorsRequest {
    const NAME: &'static str = "QueryDelegatorValidatorsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorsResponse is response type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsResponse {
    /// validators defines the validators' info of a delegator.
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDelegatorValidatorsResponse {
    const NAME: &'static str = "QueryDelegatorValidatorsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorRequest is request type for the
/// Query/DelegatorValidator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegatorValidatorRequest {
    const NAME: &'static str = "QueryDelegatorValidatorRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryDelegatorValidatorResponse response type for the
/// Query/DelegatorValidator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorResponse {
    /// validator defines the validator info.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
}
impl ::prost::Name for QueryDelegatorValidatorResponse {
    const NAME: &'static str = "QueryDelegatorValidatorResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryHistoricalInfoRequest is request type for the Query/HistoricalInfo RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHistoricalInfoRequest {
    /// height defines at which height to query the historical info.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for QueryHistoricalInfoRequest {
    const NAME: &'static str = "QueryHistoricalInfoRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryHistoricalInfoResponse is response type for the Query/HistoricalInfo RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHistoricalInfoResponse {
    /// hist defines the historical info at the given height.
    #[deprecated]
    #[prost(message, optional, tag = "1")]
    pub hist: ::core::option::Option<HistoricalInfo>,
}
impl ::prost::Name for QueryHistoricalInfoResponse {
    const NAME: &'static str = "QueryHistoricalInfoResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryPoolRequest is request type for the Query/Pool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPoolRequest {}
impl ::prost::Name for QueryPoolRequest {
    const NAME: &'static str = "QueryPoolRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryPoolResponse is response type for the Query/Pool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPoolResponse {
    /// pool defines the pool info.
    #[prost(message, optional, tag = "1")]
    pub pool: ::core::option::Option<Pool>,
}
impl ::prost::Name for QueryPoolResponse {
    const NAME: &'static str = "QueryPoolResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateValidator defines a SDK message for creating a new validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateValidator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<Description>,
    #[prost(message, optional, tag = "2")]
    pub commission: ::core::option::Option<CommissionRates>,
    #[prost(string, tag = "3")]
    pub min_self_delegation: ::prost::alloc::string::String,
    /// Deprecated: Use of Delegator Address in MsgCreateValidator is deprecated.
    /// The validator address bytes and delegator address bytes refer to the same account while creating validator (defer
    /// only in bech32 notation).
    #[deprecated]
    #[prost(string, tag = "4")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "6")]
    pub pubkey: ::core::option::Option<::pbjson_types::Any>,
    #[prost(message, optional, tag = "7")]
    pub value: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgCreateValidator {
    const NAME: &'static str = "MsgCreateValidator";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateValidatorResponse defines the Msg/CreateValidator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateValidatorResponse {}
impl ::prost::Name for MsgCreateValidatorResponse {
    const NAME: &'static str = "MsgCreateValidatorResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgEditValidator defines a SDK message for editing an existing validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditValidator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<Description>,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// We pass a reference to the new commission rate and min self delegation as
    /// it's not mandatory to update. If not updated, the deserialized rate will be
    /// zero with no way to distinguish if an update was intended.
    /// REF: #2373
    #[prost(string, tag = "3")]
    pub commission_rate: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub min_self_delegation: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgEditValidator {
    const NAME: &'static str = "MsgEditValidator";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgEditValidatorResponse defines the Msg/EditValidator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditValidatorResponse {}
impl ::prost::Name for MsgEditValidatorResponse {
    const NAME: &'static str = "MsgEditValidatorResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgDelegate defines a SDK message for performing a delegation of coins
/// from a delegator to a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgDelegate {
    const NAME: &'static str = "MsgDelegate";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgDelegateResponse defines the Msg/Delegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateResponse {}
impl ::prost::Name for MsgDelegateResponse {
    const NAME: &'static str = "MsgDelegateResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgBeginRedelegate defines a SDK message for performing a redelegation
/// of coins from a delegator and source validator to a destination validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginRedelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgBeginRedelegate {
    const NAME: &'static str = "MsgBeginRedelegate";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgBeginRedelegateResponse defines the Msg/BeginRedelegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginRedelegateResponse {
    #[prost(message, optional, tag = "1")]
    pub completion_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for MsgBeginRedelegateResponse {
    const NAME: &'static str = "MsgBeginRedelegateResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgUndelegate defines a SDK message for performing an undelegation from a
/// delegate and a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUndelegate {
    const NAME: &'static str = "MsgUndelegate";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgUndelegateResponse defines the Msg/Undelegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateResponse {
    #[prost(message, optional, tag = "1")]
    pub completion_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// amount returns the amount of undelegated coins
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgUndelegateResponse {
    const NAME: &'static str = "MsgUndelegateResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgCancelUnbondingDelegation defines the SDK message for performing a cancel unbonding delegation for delegator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUnbondingDelegation {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// amount is always less than or equal to unbonding delegation entry balance
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::base::v1beta1::Coin>,
    /// creation_height is the height which the unbonding took place.
    #[prost(int64, tag = "4")]
    pub creation_height: i64,
}
impl ::prost::Name for MsgCancelUnbondingDelegation {
    const NAME: &'static str = "MsgCancelUnbondingDelegation";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgCancelUnbondingDelegationResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUnbondingDelegationResponse {}
impl ::prost::Name for MsgCancelUnbondingDelegationResponse {
    const NAME: &'static str = "MsgCancelUnbondingDelegationResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/staking parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgRotateConsPubKey is the Msg/RotateConsPubKey request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRotateConsPubKey {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub new_pubkey: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for MsgRotateConsPubKey {
    const NAME: &'static str = "MsgRotateConsPubKey";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
/// MsgRotateConsPubKeyResponse defines the response structure for executing a
/// MsgRotateConsPubKey message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRotateConsPubKeyResponse {}
impl ::prost::Name for MsgRotateConsPubKeyResponse {
    const NAME: &'static str = "MsgRotateConsPubKeyResponse";
    const PACKAGE: &'static str = "cosmos.staking.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.staking.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
