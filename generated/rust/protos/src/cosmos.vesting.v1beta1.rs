// @generated
/// BaseVestingAccount implements the VestingAccount interface. It contains all
/// the necessary fields needed for any vesting account implementation.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseVestingAccount {
    #[prost(message, optional, tag = "1")]
    pub base_account: ::core::option::Option<super::super::auth::v1beta1::BaseAccount>,
    #[prost(message, repeated, tag = "2")]
    pub original_vesting: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "3")]
    pub delegated_free: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "4")]
    pub delegated_vesting: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// Vesting end time, as unix timestamp (in seconds).
    #[prost(int64, tag = "5")]
    pub end_time: i64,
}
impl ::prost::Name for BaseVestingAccount {
    const NAME: &'static str = "BaseVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// ContinuousVestingAccount implements the VestingAccount interface. It
/// continuously vests by unlocking coins linearly with respect to time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContinuousVestingAccount {
    #[prost(message, optional, tag = "1")]
    pub base_vesting_account: ::core::option::Option<BaseVestingAccount>,
    /// Vesting start time, as unix timestamp (in seconds).
    #[prost(int64, tag = "2")]
    pub start_time: i64,
}
impl ::prost::Name for ContinuousVestingAccount {
    const NAME: &'static str = "ContinuousVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// DelayedVestingAccount implements the VestingAccount interface. It vests all
/// coins after a specific time, but non prior. In other words, it keeps them
/// locked until a specified time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelayedVestingAccount {
    #[prost(message, optional, tag = "1")]
    pub base_vesting_account: ::core::option::Option<BaseVestingAccount>,
}
impl ::prost::Name for DelayedVestingAccount {
    const NAME: &'static str = "DelayedVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// Period defines a length of time and amount of coins that will vest.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Period {
    /// Period duration in seconds.
    #[prost(int64, tag = "1")]
    pub length: i64,
    #[prost(message, repeated, tag = "2")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for Period {
    const NAME: &'static str = "Period";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// PeriodicVestingAccount implements the VestingAccount interface. It
/// periodically vests by unlocking coins during each specified period.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeriodicVestingAccount {
    #[prost(message, optional, tag = "1")]
    pub base_vesting_account: ::core::option::Option<BaseVestingAccount>,
    #[prost(int64, tag = "2")]
    pub start_time: i64,
    #[prost(message, repeated, tag = "3")]
    pub vesting_periods: ::prost::alloc::vec::Vec<Period>,
}
impl ::prost::Name for PeriodicVestingAccount {
    const NAME: &'static str = "PeriodicVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// PermanentLockedAccount implements the VestingAccount interface. It does
/// not ever release coins, locking them indefinitely. Coins in this account can
/// still be used for delegating and for governance votes even while locked.
///
/// Since: cosmos-sdk 0.43
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PermanentLockedAccount {
    #[prost(message, optional, tag = "1")]
    pub base_vesting_account: ::core::option::Option<BaseVestingAccount>,
}
impl ::prost::Name for PermanentLockedAccount {
    const NAME: &'static str = "PermanentLockedAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateVestingAccount defines a message that enables creating a vesting
/// account.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateVestingAccount {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// end of vesting as unix time (in seconds).
    #[prost(int64, tag = "4")]
    pub end_time: i64,
    #[prost(bool, tag = "5")]
    pub delayed: bool,
}
impl ::prost::Name for MsgCreateVestingAccount {
    const NAME: &'static str = "MsgCreateVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateVestingAccountResponse defines the Msg/CreateVestingAccount response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateVestingAccountResponse {}
impl ::prost::Name for MsgCreateVestingAccountResponse {
    const NAME: &'static str = "MsgCreateVestingAccountResponse";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreatePermanentLockedAccount defines a message that enables creating a permanent
/// locked account.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePermanentLockedAccount {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgCreatePermanentLockedAccount {
    const NAME: &'static str = "MsgCreatePermanentLockedAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreatePermanentLockedAccountResponse defines the Msg/CreatePermanentLockedAccount response type.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePermanentLockedAccountResponse {}
impl ::prost::Name for MsgCreatePermanentLockedAccountResponse {
    const NAME: &'static str = "MsgCreatePermanentLockedAccountResponse";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateVestingAccount defines a message that enables creating a vesting
/// account.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePeriodicVestingAccount {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    /// start of vesting as unix time (in seconds).
    #[prost(int64, tag = "3")]
    pub start_time: i64,
    #[prost(message, repeated, tag = "4")]
    pub vesting_periods: ::prost::alloc::vec::Vec<Period>,
}
impl ::prost::Name for MsgCreatePeriodicVestingAccount {
    const NAME: &'static str = "MsgCreatePeriodicVestingAccount";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateVestingAccountResponse defines the Msg/CreatePeriodicVestingAccount
/// response type.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePeriodicVestingAccountResponse {}
impl ::prost::Name for MsgCreatePeriodicVestingAccountResponse {
    const NAME: &'static str = "MsgCreatePeriodicVestingAccountResponse";
    const PACKAGE: &'static str = "cosmos.vesting.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.vesting.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.vesting.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
