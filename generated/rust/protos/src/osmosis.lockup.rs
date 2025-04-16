// @generated
/// PeriodLock is a single lock unit by period defined by the x/lockup module.
/// It's a record of a locked coin at a specific time. It stores owner, duration,
/// unlock time and the number of coins locked. A state of a period lock is
/// created upon lock creation, and deleted once the lock has been matured after
/// the `duration` has passed since unbonding started.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeriodLock {
    /// ID is the unique id of the lock.
    /// The ID of the lock is decided upon lock creation, incrementing by 1 for
    /// every lock.
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// Owner is the account address of the lock owner.
    /// Only the owner can modify the state of the lock.
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    /// Duration is the time needed for a lock to mature after unlocking has
    /// started.
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
    /// EndTime refers to the time at which the lock would mature and get deleted.
    /// This value is first initialized when an unlock has started for the lock,
    /// end time being block time + duration.
    #[prost(message, optional, tag = "4")]
    pub end_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Coins are the tokens locked within the lock, kept in the module account.
    #[prost(message, repeated, tag = "5")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// Reward Receiver Address is the address that would be receiving rewards for
    /// the incentives for the lock. This is set to owner by default and can be
    /// changed via separate msg.
    #[prost(string, tag = "6")]
    pub reward_receiver_address: ::prost::alloc::string::String,
}
impl ::prost::Name for PeriodLock {
    const NAME: &'static str = "PeriodLock";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// QueryCondition is a struct used for querying locks upon different conditions.
/// Duration field and timestamp fields could be optional, depending on the
/// LockQueryType.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCondition {
    /// LockQueryType is a type of lock query, ByLockDuration | ByLockTime
    #[prost(enumeration = "LockQueryType", tag = "1")]
    pub lock_query_type: i32,
    /// Denom represents the token denomination we are looking to lock up
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    /// Duration is used to query locks with longer duration than the specified
    /// duration. Duration field must not be nil when the lock query type is
    /// `ByLockDuration`.
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
    /// Timestamp is used by locks started before the specified duration.
    /// Timestamp field must not be nil when the lock query type is `ByLockTime`.
    /// Querying locks with timestamp is currently not implemented.
    #[prost(message, optional, tag = "4")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for QueryCondition {
    const NAME: &'static str = "QueryCondition";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// SyntheticLock is creating virtual lockup where new denom is combination of
/// original denom and synthetic suffix. At the time of synthetic lockup creation
/// and deletion, accumulation store is also being updated and on querier side,
/// they can query as freely as native lockup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyntheticLock {
    /// Underlying Lock ID is the underlying native lock's id for this synthetic
    /// lockup. A synthetic lock MUST have an underlying lock.
    #[prost(uint64, tag = "1")]
    pub underlying_lock_id: u64,
    /// SynthDenom is the synthetic denom that is a combination of
    /// gamm share + bonding status + validator address.
    #[prost(string, tag = "2")]
    pub synth_denom: ::prost::alloc::string::String,
    /// used for unbonding synthetic lockups, for active synthetic lockups, this
    /// value is set to uninitialized value
    #[prost(message, optional, tag = "3")]
    pub end_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Duration is the duration for a synthetic lock to mature
    /// at the point of unbonding has started.
    #[prost(message, optional, tag = "4")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for SyntheticLock {
    const NAME: &'static str = "SyntheticLock";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// LockQueryType defines the type of the lock query that can
/// either be by duration or start time of the lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LockQueryType {
    ByDuration = 0,
    ByTime = 1,
    NoLock = 2,
    ByGroup = 3,
}
impl LockQueryType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            LockQueryType::ByDuration => "ByDuration",
            LockQueryType::ByTime => "ByTime",
            LockQueryType::NoLock => "NoLock",
            LockQueryType::ByGroup => "ByGroup",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ByDuration" => Some(Self::ByDuration),
            "ByTime" => Some(Self::ByTime),
            "NoLock" => Some(Self::NoLock),
            "ByGroup" => Some(Self::ByGroup),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, repeated, tag = "1")]
    pub force_unlock_allowed_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// GenesisState defines the lockup module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(uint64, tag = "1")]
    pub last_lock_id: u64,
    #[prost(message, repeated, tag = "2")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
    #[prost(message, repeated, tag = "3")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<SyntheticLock>,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleBalanceRequest {}
impl ::prost::Name for ModuleBalanceRequest {
    const NAME: &'static str = "ModuleBalanceRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleBalanceResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for ModuleBalanceResponse {
    const NAME: &'static str = "ModuleBalanceResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleLockedAmountRequest {}
impl ::prost::Name for ModuleLockedAmountRequest {
    const NAME: &'static str = "ModuleLockedAmountRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleLockedAmountResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for ModuleLockedAmountResponse {
    const NAME: &'static str = "ModuleLockedAmountResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockableCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for AccountUnlockableCoinsRequest {
    const NAME: &'static str = "AccountUnlockableCoinsRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockableCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for AccountUnlockableCoinsResponse {
    const NAME: &'static str = "AccountUnlockableCoinsResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockingCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for AccountUnlockingCoinsRequest {
    const NAME: &'static str = "AccountUnlockingCoinsRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockingCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for AccountUnlockingCoinsResponse {
    const NAME: &'static str = "AccountUnlockingCoinsResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for AccountLockedCoinsRequest {
    const NAME: &'static str = "AccountLockedCoinsRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for AccountLockedCoinsResponse {
    const NAME: &'static str = "AccountLockedCoinsResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for AccountLockedPastTimeRequest {
    const NAME: &'static str = "AccountLockedPastTimeRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedPastTimeResponse {
    const NAME: &'static str = "AccountLockedPastTimeResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeNotUnlockingOnlyRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for AccountLockedPastTimeNotUnlockingOnlyRequest {
    const NAME: &'static str = "AccountLockedPastTimeNotUnlockingOnlyRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeNotUnlockingOnlyResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedPastTimeNotUnlockingOnlyResponse {
    const NAME: &'static str = "AccountLockedPastTimeNotUnlockingOnlyResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockedBeforeTimeRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for AccountUnlockedBeforeTimeRequest {
    const NAME: &'static str = "AccountUnlockedBeforeTimeRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUnlockedBeforeTimeResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountUnlockedBeforeTimeResponse {
    const NAME: &'static str = "AccountUnlockedBeforeTimeResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeDenomRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag = "3")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for AccountLockedPastTimeDenomRequest {
    const NAME: &'static str = "AccountLockedPastTimeDenomRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedPastTimeDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedPastTimeDenomResponse {
    const NAME: &'static str = "AccountLockedPastTimeDenomResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockedDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for LockedDenomRequest {
    const NAME: &'static str = "LockedDenomRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockedDenomResponse {
    #[prost(string, tag = "1")]
    pub amount: ::prost::alloc::string::String,
}
impl ::prost::Name for LockedDenomResponse {
    const NAME: &'static str = "LockedDenomResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockedRequest {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for LockedRequest {
    const NAME: &'static str = "LockedRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockedResponse {
    #[prost(message, optional, tag = "1")]
    pub lock: ::core::option::Option<PeriodLock>,
}
impl ::prost::Name for LockedResponse {
    const NAME: &'static str = "LockedResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockRewardReceiverRequest {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for LockRewardReceiverRequest {
    const NAME: &'static str = "LockRewardReceiverRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockRewardReceiverResponse {
    #[prost(string, tag = "1")]
    pub reward_receiver: ::prost::alloc::string::String,
}
impl ::prost::Name for LockRewardReceiverResponse {
    const NAME: &'static str = "LockRewardReceiverResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextLockIdRequest {}
impl ::prost::Name for NextLockIdRequest {
    const NAME: &'static str = "NextLockIDRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextLockIdResponse {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for NextLockIdResponse {
    const NAME: &'static str = "NextLockIDResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyntheticLockupsByLockupIdRequest {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for SyntheticLockupsByLockupIdRequest {
    const NAME: &'static str = "SyntheticLockupsByLockupIDRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyntheticLockupsByLockupIdResponse {
    #[prost(message, repeated, tag = "1")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<SyntheticLock>,
}
impl ::prost::Name for SyntheticLockupsByLockupIdResponse {
    const NAME: &'static str = "SyntheticLockupsByLockupIDResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyntheticLockupByLockupIdRequest {
    #[prost(uint64, tag = "1")]
    pub lock_id: u64,
}
impl ::prost::Name for SyntheticLockupByLockupIdRequest {
    const NAME: &'static str = "SyntheticLockupByLockupIDRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyntheticLockupByLockupIdResponse {
    #[prost(message, optional, tag = "1")]
    pub synthetic_lock: ::core::option::Option<SyntheticLock>,
}
impl ::prost::Name for SyntheticLockupByLockupIdResponse {
    const NAME: &'static str = "SyntheticLockupByLockupIDResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for AccountLockedLongerDurationRequest {
    const NAME: &'static str = "AccountLockedLongerDurationRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedLongerDurationResponse {
    const NAME: &'static str = "AccountLockedLongerDurationResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedDurationRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for AccountLockedDurationRequest {
    const NAME: &'static str = "AccountLockedDurationRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedDurationResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedDurationResponse {
    const NAME: &'static str = "AccountLockedDurationResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationNotUnlockingOnlyRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for AccountLockedLongerDurationNotUnlockingOnlyRequest {
    const NAME: &'static str = "AccountLockedLongerDurationNotUnlockingOnlyRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationNotUnlockingOnlyResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedLongerDurationNotUnlockingOnlyResponse {
    const NAME: &'static str = "AccountLockedLongerDurationNotUnlockingOnlyResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationDenomRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
    #[prost(string, tag = "3")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for AccountLockedLongerDurationDenomRequest {
    const NAME: &'static str = "AccountLockedLongerDurationDenomRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountLockedLongerDurationDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for AccountLockedLongerDurationDenomResponse {
    const NAME: &'static str = "AccountLockedLongerDurationDenomResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockTokens {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgLockTokens {
    const NAME: &'static str = "MsgLockTokens";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockTokensResponse {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
impl ::prost::Name for MsgLockTokensResponse {
    const NAME: &'static str = "MsgLockTokensResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginUnlockingAll {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgBeginUnlockingAll {
    const NAME: &'static str = "MsgBeginUnlockingAll";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginUnlockingAllResponse {
    #[prost(message, repeated, tag = "1")]
    pub unlocks: ::prost::alloc::vec::Vec<PeriodLock>,
}
impl ::prost::Name for MsgBeginUnlockingAllResponse {
    const NAME: &'static str = "MsgBeginUnlockingAllResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginUnlocking {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
    /// Amount of unlocking coins. Unlock all if not set.
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgBeginUnlocking {
    const NAME: &'static str = "MsgBeginUnlocking";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginUnlockingResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(uint64, tag = "2")]
    pub unlocking_lock_id: u64,
}
impl ::prost::Name for MsgBeginUnlockingResponse {
    const NAME: &'static str = "MsgBeginUnlockingResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// MsgExtendLockup extends the existing lockup's duration.
/// The new duration is longer than the original.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExtendLockup {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
    /// duration to be set. fails if lower than the current duration, or is
    /// unlocking
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for MsgExtendLockup {
    const NAME: &'static str = "MsgExtendLockup";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExtendLockupResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgExtendLockupResponse {
    const NAME: &'static str = "MsgExtendLockupResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// MsgForceUnlock unlocks locks immediately for
/// addresses registered via governance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgForceUnlock {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
    /// Amount of unlocking coins. Unlock all if not set.
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgForceUnlock {
    const NAME: &'static str = "MsgForceUnlock";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgForceUnlockResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgForceUnlockResponse {
    const NAME: &'static str = "MsgForceUnlockResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetRewardReceiverAddress {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
    #[prost(string, tag = "3")]
    pub reward_receiver: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetRewardReceiverAddress {
    const NAME: &'static str = "MsgSetRewardReceiverAddress";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetRewardReceiverAddressResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for MsgSetRewardReceiverAddressResponse {
    const NAME: &'static str = "MsgSetRewardReceiverAddressResponse";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
/// DEPRECATED
/// Following messages are deprecated but kept to support indexing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnlockPeriodLock {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub id: u64,
}
impl ::prost::Name for MsgUnlockPeriodLock {
    const NAME: &'static str = "MsgUnlockPeriodLock";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnlockTokens {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUnlockTokens {
    const NAME: &'static str = "MsgUnlockTokens";
    const PACKAGE: &'static str = "osmosis.lockup";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.lockup.{}", Self::NAME)
    }
}
include!("osmosis.lockup.tonic.rs");
// @@protoc_insertion_point(module)
