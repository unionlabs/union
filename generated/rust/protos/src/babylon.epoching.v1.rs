// @generated
/// Epoch is a structure that contains the metadata of an epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Epoch {
    /// epoch_number is the number of this epoch
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
    /// current_epoch_interval is the epoch interval at the time of this epoch
    #[prost(uint64, tag = "2")]
    pub current_epoch_interval: u64,
    /// first_block_height is the height of the first block in this epoch
    #[prost(uint64, tag = "3")]
    pub first_block_height: u64,
    /// last_block_time is the time of the last block in this epoch.
    /// Babylon needs to remember the last header's time of each epoch to complete
    /// unbonding validators/delegations when a previous epoch's checkpoint is
    /// finalised. The last_block_time field is nil in the epoch's beginning, and
    /// is set upon the end of this epoch.
    #[prost(message, optional, tag = "4")]
    pub last_block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// sealer is the last block of the sealed epoch
    /// sealer_app_hash points to the sealer but stored in the 1st header
    /// of the next epoch
    #[prost(bytes = "vec", tag = "5")]
    pub sealer_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// sealer_block_hash is the hash of the sealer
    /// the validator set has generated a BLS multisig on the hash,
    /// i.e., hash of the last block in the epoch
    #[prost(bytes = "vec", tag = "6")]
    pub sealer_block_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Epoch {
    const NAME: &'static str = "Epoch";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueuedMessage is a message that can change the validator set and is delayed
/// to the end of an epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueuedMessage {
    /// tx_id is the ID of the tx that contains the message
    #[prost(bytes = "vec", tag = "1")]
    pub tx_id: ::prost::alloc::vec::Vec<u8>,
    /// msg_id is the original message ID, i.e., hash of the marshaled message
    #[prost(bytes = "vec", tag = "2")]
    pub msg_id: ::prost::alloc::vec::Vec<u8>,
    /// block_height is the height when this msg is submitted to Babylon
    #[prost(uint64, tag = "3")]
    pub block_height: u64,
    /// block_time is the timestamp when this msg is submitted to Babylon
    #[prost(message, optional, tag = "4")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// msg is the actual message that is sent by a user and is queued by the
    /// epoching module
    #[prost(oneof = "queued_message::Msg", tags = "5, 6, 7, 8, 9, 10, 11")]
    pub msg: ::core::option::Option<queued_message::Msg>,
}
/// Nested message and enum types in `QueuedMessage`.
pub mod queued_message {
    /// msg is the actual message that is sent by a user and is queued by the
    /// epoching module
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "5")]
        MsgCreateValidator(
            super::super::super::super::cosmos::staking::v1beta1::MsgCreateValidator,
        ),
        #[prost(message, tag = "6")]
        MsgDelegate(super::super::super::super::cosmos::staking::v1beta1::MsgDelegate),
        #[prost(message, tag = "7")]
        MsgUndelegate(super::super::super::super::cosmos::staking::v1beta1::MsgUndelegate),
        #[prost(message, tag = "8")]
        MsgBeginRedelegate(
            super::super::super::super::cosmos::staking::v1beta1::MsgBeginRedelegate,
        ),
        #[prost(message, tag = "9")]
        MsgCancelUnbondingDelegation(
            super::super::super::super::cosmos::staking::v1beta1::MsgCancelUnbondingDelegation,
        ),
        #[prost(message, tag = "10")]
        MsgEditValidator(super::super::super::super::cosmos::staking::v1beta1::MsgEditValidator),
        #[prost(message, tag = "11")]
        MsgUpdateParams(super::super::super::super::cosmos::staking::v1beta1::MsgUpdateParams),
    }
}
impl ::prost::Name for QueuedMessage {
    const NAME: &'static str = "QueuedMessage";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// ValStateUpdate is a message that records a state update of a validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValStateUpdate {
    #[prost(enumeration = "BondState", tag = "1")]
    pub state: i32,
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
    #[prost(message, optional, tag = "3")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for ValStateUpdate {
    const NAME: &'static str = "ValStateUpdate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// ValidatorLifecycle is a message that records the lifecycle of
/// a validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorLifecycle {
    #[prost(string, tag = "1")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub val_life: ::prost::alloc::vec::Vec<ValStateUpdate>,
}
impl ::prost::Name for ValidatorLifecycle {
    const NAME: &'static str = "ValidatorLifecycle";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// DelegationStateUpdate is the message that records a state update of a
/// delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationStateUpdate {
    #[prost(enumeration = "BondState", tag = "1")]
    pub state: i32,
    #[prost(string, tag = "2")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(uint64, tag = "4")]
    pub block_height: u64,
    #[prost(message, optional, tag = "5")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for DelegationStateUpdate {
    const NAME: &'static str = "DelegationStateUpdate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// ValidatorLifecycle is a message that records the lifecycle of
/// a delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationLifecycle {
    #[prost(string, tag = "1")]
    pub del_addr: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub del_life: ::prost::alloc::vec::Vec<DelegationStateUpdate>,
}
impl ::prost::Name for DelegationLifecycle {
    const NAME: &'static str = "DelegationLifecycle";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// Validator is a message that denotes a validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// addr is the validator's address (in sdk.ValAddress)
    #[prost(bytes = "vec", tag = "1")]
    pub addr: ::prost::alloc::vec::Vec<u8>,
    /// power is the validator's voting power
    #[prost(int64, tag = "2")]
    pub power: i64,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// BondState is the bond state of a validator or delegation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BondState {
    /// CREATED is when the validator/delegation has been created
    Created = 0,
    /// CREATED is when the validator/delegation has become bonded
    Bonded = 1,
    /// CREATED is when the validator/delegation has become unbonding
    Unbonding = 2,
    /// CREATED is when the validator/delegation has become unbonded
    Unbonded = 3,
    /// CREATED is when the validator/delegation has been removed
    Removed = 4,
}
impl BondState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BondState::Created => "CREATED",
            BondState::Bonded => "BONDED",
            BondState::Unbonding => "UNBONDING",
            BondState::Unbonded => "UNBONDED",
            BondState::Removed => "REMOVED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CREATED" => Some(Self::Created),
            "BONDED" => Some(Self::Bonded),
            "UNBONDING" => Some(Self::Unbonding),
            "UNBONDED" => Some(Self::Unbonded),
            "REMOVED" => Some(Self::Removed),
            _ => None,
        }
    }
}
/// EventBeginEpoch is the event emitted when an epoch has started
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBeginEpoch {
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
}
impl ::prost::Name for EventBeginEpoch {
    const NAME: &'static str = "EventBeginEpoch";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventEndEpoch is the event emitted when an epoch has ended
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventEndEpoch {
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
}
impl ::prost::Name for EventEndEpoch {
    const NAME: &'static str = "EventEndEpoch";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventHandleQueuedMsg is the event emitted when a queued message has been
/// handled
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventHandleQueuedMsg {
    #[prost(string, tag = "1")]
    pub original_event_type: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub epoch_number: u64,
    #[prost(uint64, tag = "3")]
    pub height: u64,
    #[prost(bytes = "vec", tag = "4")]
    pub tx_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub msg_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "6")]
    pub original_attributes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, tag = "7")]
    pub error: ::prost::alloc::string::String,
}
impl ::prost::Name for EventHandleQueuedMsg {
    const NAME: &'static str = "EventHandleQueuedMsg";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventSlashThreshold is the event emitted when a set of validators have been
/// slashed
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventSlashThreshold {
    #[prost(int64, tag = "1")]
    pub slashed_voting_power: i64,
    #[prost(int64, tag = "2")]
    pub total_voting_power: i64,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub slashed_validators: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for EventSlashThreshold {
    const NAME: &'static str = "EventSlashThreshold";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedDelegate is the event emitted when a MsgWrappedDelegate has been
/// queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedDelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub amount: u64,
    #[prost(string, tag = "4")]
    pub denom: ::prost::alloc::string::String,
    #[prost(uint64, tag = "5")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedDelegate {
    const NAME: &'static str = "EventWrappedDelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedUndelegate is the event emitted when a MsgWrappedUndelegate has
/// been queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedUndelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub amount: u64,
    #[prost(string, tag = "4")]
    pub denom: ::prost::alloc::string::String,
    #[prost(uint64, tag = "5")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedUndelegate {
    const NAME: &'static str = "EventWrappedUndelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedBeginRedelegate is the event emitted when a
/// MsgWrappedBeginRedelegate has been queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedBeginRedelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source_validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub destination_validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub amount: u64,
    #[prost(string, tag = "5")]
    pub denom: ::prost::alloc::string::String,
    #[prost(uint64, tag = "6")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedBeginRedelegate {
    const NAME: &'static str = "EventWrappedBeginRedelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedCancelUnbondingDelegation is the event emitted when a
/// MsgWrappedCancelUnbondingDelegation has been queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedCancelUnbondingDelegation {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub amount: u64,
    #[prost(int64, tag = "4")]
    pub creation_height: i64,
    #[prost(uint64, tag = "5")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedCancelUnbondingDelegation {
    const NAME: &'static str = "EventWrappedCancelUnbondingDelegation";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedEditValidator is the event emitted when a
/// MsgWrappedEditValidator has been queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedEditValidator {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedEditValidator {
    const NAME: &'static str = "EventWrappedEditValidator";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EventWrappedStakingUpdateParams is the event emitted when a
/// MsgWrappedStakingUpdateParams has been queued
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventWrappedStakingUpdateParams {
    /// unbonding_time is the time duration of unbonding.
    #[prost(string, tag = "1")]
    pub unbonding_time: ::prost::alloc::string::String,
    /// max_validators is the maximum number of validators.
    #[prost(uint32, tag = "2")]
    pub max_validators: u32,
    /// max_entries is the max entries for either unbonding delegation or redelegation (per pair/trio).
    #[prost(uint32, tag = "3")]
    pub max_entries: u32,
    /// historical_entries is the number of historical entries to persist.
    #[prost(uint32, tag = "4")]
    pub historical_entries: u32,
    /// bond_denom defines the bondable coin denomination.
    #[prost(string, tag = "5")]
    pub bond_denom: ::prost::alloc::string::String,
    /// min_commission_rate is the chain-wide minimum commission rate that a validator can charge their delegators
    #[prost(string, tag = "6")]
    pub min_commission_rate: ::prost::alloc::string::String,
    #[prost(uint64, tag = "7")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for EventWrappedStakingUpdateParams {
    const NAME: &'static str = "EventWrappedStakingUpdateParams";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// epoch_interval is the number of consecutive blocks to form an epoch
    #[prost(uint64, tag = "1")]
    pub epoch_interval: u64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the epoching module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochInfoRequest is the request type for the Query/EpochInfo method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochInfoRequest {
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryEpochInfoRequest {
    const NAME: &'static str = "QueryEpochInfoRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochInfoRequest is the response type for the Query/EpochInfo method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub epoch: ::core::option::Option<EpochResponse>,
}
impl ::prost::Name for QueryEpochInfoResponse {
    const NAME: &'static str = "QueryEpochInfoResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochInfosRequest is the request type for the Query/EpochInfos method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochsInfoRequest {
    /// pagination defines whether to have the pagination in the request
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryEpochsInfoRequest {
    const NAME: &'static str = "QueryEpochsInfoRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochsInfoResponse is the response type for the Query/EpochInfos method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochsInfoResponse {
    #[prost(message, repeated, tag = "1")]
    pub epochs: ::prost::alloc::vec::Vec<EpochResponse>,
    /// pagination defines the pagination in the response
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryEpochsInfoResponse {
    const NAME: &'static str = "QueryEpochsInfoResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryCurrentEpochRequest is the request type for the Query/CurrentEpoch RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCurrentEpochRequest {}
impl ::prost::Name for QueryCurrentEpochRequest {
    const NAME: &'static str = "QueryCurrentEpochRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryCurrentEpochResponse is the response type for the Query/CurrentEpoch RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCurrentEpochResponse {
    /// current_epoch is the current epoch number
    #[prost(uint64, tag = "1")]
    pub current_epoch: u64,
    /// epoch_boundary is the height of this epoch's last block
    #[prost(uint64, tag = "2")]
    pub epoch_boundary: u64,
}
impl ::prost::Name for QueryCurrentEpochResponse {
    const NAME: &'static str = "QueryCurrentEpochResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochMsgsRequest is the request type for the Query/EpochMsgs RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochMsgsRequest {
    /// epoch_num is the number of epoch of the requested msg queue
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    /// pagination defines whether to have the pagination in the request
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryEpochMsgsRequest {
    const NAME: &'static str = "QueryEpochMsgsRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochMsgsResponse is the response type for the Query/EpochMsgs RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochMsgsResponse {
    /// msgs is the list of messages queued in the current epoch
    #[prost(message, repeated, tag = "1")]
    pub msgs: ::prost::alloc::vec::Vec<QueuedMessageResponse>,
    /// pagination defines the pagination in the response
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryEpochMsgsResponse {
    const NAME: &'static str = "QueryEpochMsgsResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryLatestEpochMsgsRequest is the request type for the Query/LatestEpochMsgs
/// RPC method it returns epoch msgs within epoch [max(1,
/// end_epoch-epoch_count+1), end_epoch]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLatestEpochMsgsRequest {
    /// end_epoch is the number of the last epoch to query
    #[prost(uint64, tag = "1")]
    pub end_epoch: u64,
    /// epoch_count is the number of epochs to query
    #[prost(uint64, tag = "2")]
    pub epoch_count: u64,
    #[prost(message, optional, tag = "3")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryLatestEpochMsgsRequest {
    const NAME: &'static str = "QueryLatestEpochMsgsRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryLatestEpochMsgsResponse is the response type for the
/// Query/LatestEpochMsgs RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLatestEpochMsgsResponse {
    /// latest_epoch_msgs is a list of QueuedMessageList
    /// each QueuedMessageList has a field identifying the epoch number
    #[prost(message, repeated, tag = "1")]
    pub latest_epoch_msgs: ::prost::alloc::vec::Vec<QueuedMessageList>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryLatestEpochMsgsResponse {
    const NAME: &'static str = "QueryLatestEpochMsgsResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryValidatorLifecycleRequest is the request type for the
/// Query/ValidatorLifecycle RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorLifecycleRequest {
    #[prost(string, tag = "1")]
    pub val_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorLifecycleRequest {
    const NAME: &'static str = "QueryValidatorLifecycleRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryValidatorLifecycleResponse is the response type for the
/// Query/ValidatorLifecycle RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorLifecycleResponse {
    #[prost(string, tag = "1")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub val_life: ::prost::alloc::vec::Vec<ValStateUpdateResponse>,
}
impl ::prost::Name for QueryValidatorLifecycleResponse {
    const NAME: &'static str = "QueryValidatorLifecycleResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryDelegationLifecycleRequest is the request type for the
/// Query/DelegationLifecycle RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationLifecycleRequest {
    #[prost(string, tag = "1")]
    pub del_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDelegationLifecycleRequest {
    const NAME: &'static str = "QueryDelegationLifecycleRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryDelegationLifecycleRequest is the response type for the
/// Query/DelegationLifecycle RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationLifecycleResponse {
    #[prost(message, optional, tag = "1")]
    pub del_life: ::core::option::Option<DelegationLifecycle>,
}
impl ::prost::Name for QueryDelegationLifecycleResponse {
    const NAME: &'static str = "QueryDelegationLifecycleResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochValSetRequest is the request type for the Query/EpochValSet RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochValSetRequest {
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryEpochValSetRequest {
    const NAME: &'static str = "QueryEpochValSetRequest";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueryEpochValSetRequest is the response type for the Query/EpochValSet RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochValSetResponse {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(int64, tag = "2")]
    pub total_voting_power: i64,
    #[prost(message, optional, tag = "3")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryEpochValSetResponse {
    const NAME: &'static str = "QueryEpochValSetResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// EpochResponse is a structure that contains the metadata of an epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochResponse {
    /// epoch_number is the number of this epoch
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
    /// current_epoch_interval is the epoch interval at the time of this epoch
    #[prost(uint64, tag = "2")]
    pub current_epoch_interval: u64,
    /// first_block_height is the height of the first block in this epoch
    #[prost(uint64, tag = "3")]
    pub first_block_height: u64,
    /// last_block_time is the time of the last block in this epoch.
    /// Babylon needs to remember the last header's time of each epoch to complete
    /// unbonding validators/delegations when a previous epoch's checkpoint is
    /// finalised. The last_block_time field is nil in the epoch's beginning, and
    /// is set upon the end of this epoch.
    #[prost(message, optional, tag = "4")]
    pub last_block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// sealer is the last block of the sealed epoch
    /// sealer_app_hash points to the sealer but stored in the 1st header
    /// of the next epoch as hex string.
    #[prost(string, tag = "5")]
    pub sealer_app_hash_hex: ::prost::alloc::string::String,
    /// sealer_block_hash is the hash of the sealer
    /// the validator set has generated a BLS multisig on the hash,
    /// i.e., hash of the last block in the epoch as hex string.
    #[prost(string, tag = "6")]
    pub sealer_block_hash: ::prost::alloc::string::String,
}
impl ::prost::Name for EpochResponse {
    const NAME: &'static str = "EpochResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueuedMessageResponse is a message that can change the validator set and is delayed
/// to the end of an epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueuedMessageResponse {
    /// tx_id is the ID of the tx that contains the message as hex.
    #[prost(string, tag = "1")]
    pub tx_id: ::prost::alloc::string::String,
    /// msg_id is the original message ID, i.e., hash of the marshaled message as hex.
    #[prost(string, tag = "2")]
    pub msg_id: ::prost::alloc::string::String,
    /// block_height is the height when this msg is submitted to Babylon
    #[prost(uint64, tag = "3")]
    pub block_height: u64,
    /// block_time is the timestamp when this msg is submitted to Babylon
    #[prost(message, optional, tag = "4")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// msg is the actual message that is sent by a user and is queued by the
    /// epoching module as string.
    #[prost(string, tag = "5")]
    pub msg: ::prost::alloc::string::String,
}
impl ::prost::Name for QueuedMessageResponse {
    const NAME: &'static str = "QueuedMessageResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// QueuedMessageList is a message that contains a list of staking-related
/// messages queued for an epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueuedMessageList {
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<QueuedMessageResponse>,
}
impl ::prost::Name for QueuedMessageList {
    const NAME: &'static str = "QueuedMessageList";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// ValStateUpdateResponse is a message response that records a state update of a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValStateUpdateResponse {
    /// StateDesc defines the descriptive state.
    #[prost(string, tag = "1")]
    pub state_desc: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
    #[prost(message, optional, tag = "3")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for ValStateUpdateResponse {
    const NAME: &'static str = "ValStateUpdateResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedDelegate is the message for delegating stakes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedDelegate {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgDelegate>,
}
impl ::prost::Name for MsgWrappedDelegate {
    const NAME: &'static str = "MsgWrappedDelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedDelegate is the response to the MsgWrappedDelegate message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedDelegateResponse {}
impl ::prost::Name for MsgWrappedDelegateResponse {
    const NAME: &'static str = "MsgWrappedDelegateResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedUndelegate is the message for undelegating stakes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedUndelegate {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgUndelegate>,
}
impl ::prost::Name for MsgWrappedUndelegate {
    const NAME: &'static str = "MsgWrappedUndelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedUndelegateResponse is the response to the MsgWrappedUndelegate
/// message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedUndelegateResponse {}
impl ::prost::Name for MsgWrappedUndelegateResponse {
    const NAME: &'static str = "MsgWrappedUndelegateResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedDelegate is the message for moving bonded stakes from a
/// validator to another validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedBeginRedelegate {
    #[prost(message, optional, tag = "1")]
    pub msg:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgBeginRedelegate>,
}
impl ::prost::Name for MsgWrappedBeginRedelegate {
    const NAME: &'static str = "MsgWrappedBeginRedelegate";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedBeginRedelegateResponse is the response to the
/// MsgWrappedBeginRedelegate message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedBeginRedelegateResponse {}
impl ::prost::Name for MsgWrappedBeginRedelegateResponse {
    const NAME: &'static str = "MsgWrappedBeginRedelegateResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedCancelUnbondingDelegation is the message for cancelling
/// an unbonding delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedCancelUnbondingDelegation {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<
        super::super::super::cosmos::staking::v1beta1::MsgCancelUnbondingDelegation,
    >,
}
impl ::prost::Name for MsgWrappedCancelUnbondingDelegation {
    const NAME: &'static str = "MsgWrappedCancelUnbondingDelegation";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedCancelUnbondingDelegationResponse is the response to the
/// MsgWrappedCancelUnbondingDelegation message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedCancelUnbondingDelegationResponse {}
impl ::prost::Name for MsgWrappedCancelUnbondingDelegationResponse {
    const NAME: &'static str = "MsgWrappedCancelUnbondingDelegationResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedEditValidator defines a message for updating validator description
/// and commission rate.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedEditValidator {
    #[prost(message, optional, tag = "1")]
    pub msg:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgEditValidator>,
}
impl ::prost::Name for MsgWrappedEditValidator {
    const NAME: &'static str = "MsgWrappedEditValidator";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedEditValidatorResponse is the response to the MsgWrappedEditValidator message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedEditValidatorResponse {}
impl ::prost::Name for MsgWrappedEditValidatorResponse {
    const NAME: &'static str = "MsgWrappedEditValidatorResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedStakingUpdateParams defines a message for updating x/staking module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedStakingUpdateParams {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgUpdateParams>,
}
impl ::prost::Name for MsgWrappedStakingUpdateParams {
    const NAME: &'static str = "MsgWrappedStakingUpdateParams";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgWrappedStakingUpdateParamsResponse is the response to the MsgWrappedStakingUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedStakingUpdateParamsResponse {}
impl ::prost::Name for MsgWrappedStakingUpdateParamsResponse {
    const NAME: &'static str = "MsgWrappedStakingUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message for updating epoching module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet implemented
    /// in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the epoching parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse is the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.epoching.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.epoching.v1.{}", Self::NAME)
    }
}
include!("babylon.epoching.v1.tonic.rs");
// @@protoc_insertion_point(module)
