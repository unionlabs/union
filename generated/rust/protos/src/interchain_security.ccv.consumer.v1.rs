// @generated
/// CrossChainValidator defines the type used to store validator information
/// internal to the consumer CCV module.  Note one cross chain validator entry is
/// persisted for each consumer validator, where incoming VSC packets update this
/// data, which is eventually forwarded to comet for consumer chain consensus.
///
/// Note this type is only used internally to the consumer CCV module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CrossChainValidator {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "2")]
    pub power: i64,
    /// pubkey is the consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "3")]
    pub pubkey: ::core::option::Option<::pbjson_types::Any>,
    /// !!! DEPRECATED !!! opted_out is deprecated because after the introduction of Partial Set Security (PSS)
    /// we removed the soft opt-out feature.
    #[deprecated]
    #[prost(bool, tag = "4")]
    pub opted_out: bool,
}
impl ::prost::Name for CrossChainValidator {
    const NAME: &'static str = "CrossChainValidator";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// A record storing the state of a slash packet sent to the provider chain
/// which may bounce back and forth until handled by the provider.
///
/// Note this type is only used internally to the consumer CCV module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashRecord {
    #[prost(bool, tag = "1")]
    pub waiting_on_reply: bool,
    #[prost(message, optional, tag = "2")]
    pub send_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for SlashRecord {
    const NAME: &'static str = "SlashRecord";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the CCV consumer genesis state
///
/// Note: this type is only used on consumer side and references shared types with
/// provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// ConsumerParams is a shared type with provider module
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<super::super::v1::ConsumerParams>,
    /// Client ID of the provider. Empty for a new chain, filled in on restart.
    #[prost(string, tag = "2")]
    pub provider_client_id: ::prost::alloc::string::String,
    /// Channel ID of the provider. Empty for a new chain, filled in on restart.
    #[prost(string, tag = "3")]
    pub provider_channel_id: ::prost::alloc::string::String,
    /// true for new chain, false for chain restart.
    #[prost(bool, tag = "4")]
    pub new_chain: bool,
    /// HeightToValsetUpdateId nil on new chain, filled in on restart.
    #[prost(message, repeated, tag = "9")]
    pub height_to_valset_update_id: ::prost::alloc::vec::Vec<HeightToValsetUpdateId>,
    /// OutstandingDowntimes nil on new chain, filled  in on restart.
    #[prost(message, repeated, tag = "10")]
    pub outstanding_downtime_slashing: ::prost::alloc::vec::Vec<OutstandingDowntime>,
    /// PendingConsumerPackets nil on new chain, filled in on restart.
    #[prost(message, optional, tag = "11")]
    pub pending_consumer_packets: ::core::option::Option<ConsumerPacketDataList>,
    /// LastTransmissionBlockHeight nil on new chain, filled in on restart.
    #[prost(message, optional, tag = "12")]
    pub last_transmission_block_height: ::core::option::Option<LastTransmissionBlockHeight>,
    /// Flag indicating whether the consumer CCV module starts in pre-CCV state
    #[prost(bool, tag = "13")]
    pub pre_ccv: bool,
    #[prost(message, optional, tag = "14")]
    pub provider: ::core::option::Option<super::super::v1::ProviderInfo>,
    /// The ID of the connection end on the consumer chain on top of which the
    /// CCV channel will be established. If connection_id == "", a new client of
    /// the provider chain and a new connection on top of this client are created.
    /// The new client is initialized using provider.client_state and provider.consensus_state.
    #[prost(string, tag = "15")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// HeightValsetUpdateID represents a mapping internal to the consumer CCV module
/// which links a block height to each recv valset update id.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeightToValsetUpdateId {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint64, tag = "2")]
    pub valset_update_id: u64,
}
impl ::prost::Name for HeightToValsetUpdateId {
    const NAME: &'static str = "HeightToValsetUpdateID";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// OutstandingDowntime defines the type used internally to the consumer CCV
/// module and is used in order to not send multiple slashing requests for
/// the same downtime infraction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutstandingDowntime {
    #[prost(string, tag = "1")]
    pub validator_consensus_address: ::prost::alloc::string::String,
}
impl ::prost::Name for OutstandingDowntime {
    const NAME: &'static str = "OutstandingDowntime";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// LastTransmissionBlockHeight is the last time validator holding
/// pools were transmitted to the provider chain. This type is used internally
/// to the consumer CCV module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastTransmissionBlockHeight {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for LastTransmissionBlockHeight {
    const NAME: &'static str = "LastTransmissionBlockHeight";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// ConsumerPacketDataList is a list of consumer packet data packets.
///
/// Note this type is used internally to the consumer CCV module
/// for exporting / importing state in InitGenesis and ExportGenesis.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerPacketDataList {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<super::super::v1::ConsumerPacketData>,
}
impl ::prost::Name for ConsumerPacketDataList {
    const NAME: &'static str = "ConsumerPacketDataList";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// NextFeeDistributionEstimate holds information about next fee distribution
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextFeeDistributionEstimate {
    /// current block height at the time of querying
    #[prost(int64, tag = "1")]
    pub current_height: i64,
    /// block height at which last distribution took place
    #[prost(int64, tag = "2")]
    pub last_height: i64,
    /// block height at which next distribution will take place
    #[prost(int64, tag = "3")]
    pub next_height: i64,
    /// ratio between consumer and provider fee distribution
    #[prost(string, tag = "4")]
    pub distribution_fraction: ::prost::alloc::string::String,
    /// total accruead fees at the time of querying
    #[prost(string, tag = "5")]
    pub total: ::prost::alloc::string::String,
    /// amount distributed to provider chain
    #[prost(string, tag = "6")]
    pub to_provider: ::prost::alloc::string::String,
    /// amount distributed (kept) by consumer chain
    #[prost(string, tag = "7")]
    pub to_consumer: ::prost::alloc::string::String,
}
impl ::prost::Name for NextFeeDistributionEstimate {
    const NAME: &'static str = "NextFeeDistributionEstimate";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextFeeDistributionEstimateRequest {}
impl ::prost::Name for QueryNextFeeDistributionEstimateRequest {
    const NAME: &'static str = "QueryNextFeeDistributionEstimateRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextFeeDistributionEstimateResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<NextFeeDistributionEstimate>,
}
impl ::prost::Name for QueryNextFeeDistributionEstimateResponse {
    const NAME: &'static str = "QueryNextFeeDistributionEstimateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<super::super::v1::ConsumerParams>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProviderInfoRequest {}
impl ::prost::Name for QueryProviderInfoRequest {
    const NAME: &'static str = "QueryProviderInfoRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProviderInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub consumer: ::core::option::Option<ChainInfo>,
    #[prost(message, optional, tag = "2")]
    pub provider: ::core::option::Option<ChainInfo>,
}
impl ::prost::Name for QueryProviderInfoResponse {
    const NAME: &'static str = "QueryProviderInfoResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateRequest {}
impl ::prost::Name for QueryThrottleStateRequest {
    const NAME: &'static str = "QueryThrottleStateRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateResponse {
    #[prost(message, optional, tag = "1")]
    pub slash_record: ::core::option::Option<SlashRecord>,
    #[prost(message, repeated, tag = "2")]
    pub packet_data_queue: ::prost::alloc::vec::Vec<super::super::v1::ConsumerPacketData>,
}
impl ::prost::Name for QueryThrottleStateResponse {
    const NAME: &'static str = "QueryThrottleStateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainInfo {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub channel_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ChainInfo {
    const NAME: &'static str = "ChainInfo";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/provider parameters to update.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<super::super::v1::ConsumerParams>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.consumer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.consumer.v1.{}", Self::NAME)
    }
}
include!("interchain_security.ccv.consumer.v1.tonic.rs");
// @@protoc_insertion_point(module)
