// @generated
/// A TWAP record should be indexed in state by pool_id, (asset pair), timestamp
/// The asset pair assets should be lexicographically sorted.
/// Technically (pool_id, asset_0_denom, asset_1_denom, height) do not need to
/// appear in the struct however we view this as the wrong performance tradeoff
/// given SDK today. Would rather we optimize for readability and correctness,
/// than an optimal state storage format. The system bottleneck is elsewhere for
/// now.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TwapRecord {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    /// Lexicographically smaller denom of the pair
    #[prost(string, tag = "2")]
    pub asset0_denom: ::prost::alloc::string::String,
    /// Lexicographically larger denom of the pair
    #[prost(string, tag = "3")]
    pub asset1_denom: ::prost::alloc::string::String,
    /// height this record corresponds to, for debugging purposes
    #[prost(int64, tag = "4")]
    pub height: i64,
    /// This field should only exist until we have a global registry in the state
    /// machine, mapping prior block heights within {TIME RANGE} to times.
    #[prost(message, optional, tag = "5")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// We store the last spot prices in the struct, so that we can interpolate
    /// accumulator values for times between when accumulator records are stored.
    #[prost(string, tag = "6")]
    pub p0_last_spot_price: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub p1_last_spot_price: ::prost::alloc::string::String,
    #[prost(string, tag = "8")]
    pub p0_arithmetic_twap_accumulator: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub p1_arithmetic_twap_accumulator: ::prost::alloc::string::String,
    #[prost(string, tag = "10")]
    pub geometric_twap_accumulator: ::prost::alloc::string::String,
    /// This field contains the time in which the last spot price error occurred.
    /// It is used to alert the caller if they are getting a potentially erroneous
    /// TWAP, due to an unforeseen underlying error.
    #[prost(message, optional, tag = "11")]
    pub last_error_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for TwapRecord {
    const NAME: &'static str = "TwapRecord";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
/// PruningState allows us to spread out the pruning of TWAP records over time,
/// instead of pruning all at once at the end of the epoch.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PruningState {
    /// is_pruning is true if the pruning process is ongoing.
    /// This tells the module to continue pruning the TWAP records
    /// at the EndBlock.
    #[prost(bool, tag = "1")]
    pub is_pruning: bool,
    /// last_kept_time is the time of the last kept TWAP record.
    /// This is used to determine all TWAP records that are older than
    /// last_kept_time and should be pruned.
    #[prost(message, optional, tag = "2")]
    pub last_kept_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Deprecated: This field is deprecated.
    #[deprecated]
    #[prost(bytes = "vec", tag = "3")]
    pub last_key_seen: ::prost::alloc::vec::Vec<u8>,
    /// last_seen_pool_id is the pool_id that we will begin pruning in the next
    /// block. This value starts at the highest pool_id at time of epoch, and
    /// decreases until it reaches 1. When it reaches 1, the pruning
    /// process is complete.
    #[prost(uint64, tag = "4")]
    pub last_seen_pool_id: u64,
}
impl ::prost::Name for PruningState {
    const NAME: &'static str = "PruningState";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
/// Params holds parameters for the twap module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, tag = "1")]
    pub prune_epoch_identifier: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub record_history_keep_period: ::core::option::Option<::pbjson_types::Duration>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the twap module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// twaps is the collection of all twap records.
    #[prost(message, repeated, tag = "1")]
    pub twaps: ::prost::alloc::vec::Vec<TwapRecord>,
    /// params is the container of twap parameters.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub start_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(message, optional, tag = "5")]
    pub end_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for ArithmeticTwapRequest {
    const NAME: &'static str = "ArithmeticTwapRequest";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapResponse {
    #[prost(string, tag = "1")]
    pub arithmetic_twap: ::prost::alloc::string::String,
}
impl ::prost::Name for ArithmeticTwapResponse {
    const NAME: &'static str = "ArithmeticTwapResponse";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapToNowRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub start_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for ArithmeticTwapToNowRequest {
    const NAME: &'static str = "ArithmeticTwapToNowRequest";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapToNowResponse {
    #[prost(string, tag = "1")]
    pub arithmetic_twap: ::prost::alloc::string::String,
}
impl ::prost::Name for ArithmeticTwapToNowResponse {
    const NAME: &'static str = "ArithmeticTwapToNowResponse";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeometricTwapRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub start_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(message, optional, tag = "5")]
    pub end_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for GeometricTwapRequest {
    const NAME: &'static str = "GeometricTwapRequest";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeometricTwapResponse {
    #[prost(string, tag = "1")]
    pub geometric_twap: ::prost::alloc::string::String,
}
impl ::prost::Name for GeometricTwapResponse {
    const NAME: &'static str = "GeometricTwapResponse";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeometricTwapToNowRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub start_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for GeometricTwapToNowRequest {
    const NAME: &'static str = "GeometricTwapToNowRequest";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeometricTwapToNowResponse {
    #[prost(string, tag = "1")]
    pub geometric_twap: ::prost::alloc::string::String,
}
impl ::prost::Name for GeometricTwapToNowResponse {
    const NAME: &'static str = "GeometricTwapToNowResponse";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsRequest {}
impl ::prost::Name for ParamsRequest {
    const NAME: &'static str = "ParamsRequest";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for ParamsResponse {
    const NAME: &'static str = "ParamsResponse";
    const PACKAGE: &'static str = "osmosis.twap.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.twap.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.twap.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
