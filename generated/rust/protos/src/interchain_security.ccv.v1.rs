// @generated
/// ConsumerParams defines the parameters for CCV consumer module.
///
/// Note this type is referenced in both the consumer and provider CCV modules,
/// and persisted on the provider, see MakeConsumerGenesis and
/// SetConsumerGenesis.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerParams {
    /// TODO: Remove enabled flag and find a better way to setup integration tests
    /// See: <https://github.com/cosmos/interchain-security/issues/339>
    #[prost(bool, tag = "1")]
    pub enabled: bool,
    /// /////////////////////
    /// Distribution Params
    /// Number of blocks between ibc-token-transfers from the consumer chain to
    /// the provider chain. Note that at this transmission event a fraction of
    /// the accumulated tokens are divided and sent consumer redistribution
    /// address.
    #[prost(int64, tag = "2")]
    pub blocks_per_distribution_transmission: i64,
    /// Channel, and provider-chain receiving address to send distribution token
    /// transfers over. These parameters is auto-set during the consumer <->
    /// provider handshake procedure.
    #[prost(string, tag = "3")]
    pub distribution_transmission_channel: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub provider_fee_pool_addr_str: ::prost::alloc::string::String,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "5")]
    pub ccv_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "6")]
    pub transfer_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "7")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "8")]
    pub historical_entries: i64,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "9")]
    pub unbonding_period: ::core::option::Option<::pbjson_types::Duration>,
    /// !!! DEPRECATED !!! soft_opt_out_threshold is deprecated. see docs/docs/adrs/adr-015-partial-set-security.md
    #[deprecated]
    #[prost(string, tag = "10")]
    pub soft_opt_out_threshold: ::prost::alloc::string::String,
    /// Reward denoms. These are the denominations which are allowed to be sent to
    /// the provider as rewards.
    #[prost(string, repeated, tag = "11")]
    pub reward_denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Provider-originated reward denoms. These are denoms coming from the
    /// provider which are allowed to be used as rewards. e.g. "uatom"
    #[prost(string, repeated, tag = "12")]
    pub provider_reward_denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The period after which a consumer can retry sending a throttled packet.
    #[prost(message, optional, tag = "13")]
    pub retry_delay_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The consumer ID of this consumer chain. Used by the consumer module to send
    /// ICS rewards.
    #[prost(string, tag = "14")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ConsumerParams {
    const NAME: &'static str = "ConsumerParams";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// ConsumerGenesisState defines shared genesis information between provider and
/// consumer
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerGenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<ConsumerParams>,
    #[prost(message, optional, tag = "2")]
    pub provider: ::core::option::Option<ProviderInfo>,
    /// True for new chain, false for chain restart.
    /// This is needed and always set to true; otherwise, new_chain in the consumer
    /// genesis state will default to false
    #[prost(bool, tag = "3")]
    pub new_chain: bool,
    /// Flag indicating whether the consumer CCV module starts in pre-CCV state
    #[prost(bool, tag = "4")]
    pub pre_ccv: bool,
    /// The ID of the connection end on the consumer chain on top of which the
    /// CCV channel will be established. If connection_id == "", a new client of
    /// the provider chain and a new connection on top of this client are created.
    /// The new client is initialized using client_state and consensus_state.
    #[prost(string, tag = "5")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ConsumerGenesisState {
    const NAME: &'static str = "ConsumerGenesisState";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// ProviderInfo defines all information a consumer needs from a provider
/// Shared data type between provider and consumer
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProviderInfo {
    /// The client state for the provider client filled in on new chain, nil on restart.
    /// If connection_id != "", then client_state is ignored.
    #[prost(message, optional, tag = "1")]
    pub client_state:
        ::core::option::Option<super::super::super::ibc::lightclients::tendermint::v1::ClientState>,
    /// The consensus state for the provider client filled in on new chain, nil on restart.
    /// If connection_id != "", then consensus_state is ignored.
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<
        super::super::super::ibc::lightclients::tendermint::v1::ConsensusState,
    >,
    /// InitialValset filled in on new chain and on restart.
    #[prost(message, repeated, tag = "3")]
    pub initial_val_set:
        ::prost::alloc::vec::Vec<super::super::super::tendermint::abci::ValidatorUpdate>,
}
impl ::prost::Name for ProviderInfo {
    const NAME: &'static str = "ProviderInfo";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// This packet is sent from provider chain to consumer chain if the validator
/// set for consumer chain changes (due to new bonding/unbonding messages or
/// slashing events) A VSCMatured packet from consumer chain will be sent
/// asynchronously once unbonding period is over, and this will function as
/// `UnbondingOver` message for this packet.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetChangePacketData {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates:
        ::prost::alloc::vec::Vec<super::super::super::tendermint::abci::ValidatorUpdate>,
    #[prost(uint64, tag = "2")]
    pub valset_update_id: u64,
    /// consensus address of consumer chain validators
    /// successfully slashed on the provider chain
    #[prost(string, repeated, tag = "3")]
    pub slash_acks: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ValidatorSetChangePacketData {
    const NAME: &'static str = "ValidatorSetChangePacketData";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// This packet is sent from the consumer chain to the provider chain
/// to notify that a VSC packet reached maturity on the consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VscMaturedPacketData {
    /// the id of the VSC packet that reached maturity
    #[prost(uint64, tag = "1")]
    pub valset_update_id: u64,
}
impl ::prost::Name for VscMaturedPacketData {
    const NAME: &'static str = "VSCMaturedPacketData";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// This packet is sent from the consumer chain to the provider chain
/// to request the slashing of a validator as a result of an infraction
/// committed on the consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashPacketData {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::super::super::tendermint::abci::Validator>,
    /// map to the infraction block height on the provider
    #[prost(uint64, tag = "2")]
    pub valset_update_id: u64,
    /// tell if the slashing is for a downtime or a double-signing infraction
    #[prost(
        enumeration = "super::super::super::cosmos::staking::v1beta1::Infraction",
        tag = "3"
    )]
    pub infraction: i32,
}
impl ::prost::Name for SlashPacketData {
    const NAME: &'static str = "SlashPacketData";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// ConsumerPacketData contains a consumer packet data and a type tag
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerPacketData {
    #[prost(enumeration = "ConsumerPacketDataType", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "consumer_packet_data::Data", tags = "2, 3")]
    pub data: ::core::option::Option<consumer_packet_data::Data>,
}
/// Nested message and enum types in `ConsumerPacketData`.
pub mod consumer_packet_data {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "2")]
        SlashPacketData(super::SlashPacketData),
        #[prost(message, tag = "3")]
        VscMaturedPacketData(super::VscMaturedPacketData),
    }
}
impl ::prost::Name for ConsumerPacketData {
    const NAME: &'static str = "ConsumerPacketData";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// Note this type is used during IBC handshake methods for both the consumer and provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HandshakeMetadata {
    #[prost(string, tag = "1")]
    pub provider_fee_pool_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
}
impl ::prost::Name for HandshakeMetadata {
    const NAME: &'static str = "HandshakeMetadata";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// ConsumerPacketData contains a consumer packet data and a type tag
/// that is compatible with ICS v1 and v2 over the wire. It is not used for internal storage.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerPacketDataV1 {
    #[prost(enumeration = "ConsumerPacketDataType", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "consumer_packet_data_v1::Data", tags = "2, 3")]
    pub data: ::core::option::Option<consumer_packet_data_v1::Data>,
}
/// Nested message and enum types in `ConsumerPacketDataV1`.
pub mod consumer_packet_data_v1 {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "2")]
        SlashPacketData(super::SlashPacketDataV1),
        #[prost(message, tag = "3")]
        VscMaturedPacketData(super::VscMaturedPacketData),
    }
}
impl ::prost::Name for ConsumerPacketDataV1 {
    const NAME: &'static str = "ConsumerPacketDataV1";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// This packet is sent from the consumer chain to the provider chain
/// It is backward compatible with the ICS v1 and v2 version of the packet.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashPacketDataV1 {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::super::super::tendermint::abci::Validator>,
    /// map to the infraction block height on the provider
    #[prost(uint64, tag = "2")]
    pub valset_update_id: u64,
    /// tell if the slashing is for a downtime or a double-signing infraction
    #[prost(enumeration = "InfractionType", tag = "3")]
    pub infraction: i32,
}
impl ::prost::Name for SlashPacketDataV1 {
    const NAME: &'static str = "SlashPacketDataV1";
    const PACKAGE: &'static str = "interchain_security.ccv.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.v1.{}", Self::NAME)
    }
}
/// ConsumerPacketType indicates interchain security specific packet types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ConsumerPacketDataType {
    /// UNSPECIFIED packet type
    ConsumerPacketTypeUnspecified = 0,
    /// Slash packet
    ConsumerPacketTypeSlash = 1,
    /// VSCMatured packet
    ConsumerPacketTypeVscm = 2,
}
impl ConsumerPacketDataType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ConsumerPacketDataType::ConsumerPacketTypeUnspecified => {
                "CONSUMER_PACKET_TYPE_UNSPECIFIED"
            }
            ConsumerPacketDataType::ConsumerPacketTypeSlash => "CONSUMER_PACKET_TYPE_SLASH",
            ConsumerPacketDataType::ConsumerPacketTypeVscm => "CONSUMER_PACKET_TYPE_VSCM",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CONSUMER_PACKET_TYPE_UNSPECIFIED" => Some(Self::ConsumerPacketTypeUnspecified),
            "CONSUMER_PACKET_TYPE_SLASH" => Some(Self::ConsumerPacketTypeSlash),
            "CONSUMER_PACKET_TYPE_VSCM" => Some(Self::ConsumerPacketTypeVscm),
            _ => None,
        }
    }
}
/// InfractionType indicates the infraction type a validator committed.
/// Note ccv.InfractionType to maintain compatibility between ICS versions
/// using different versions of the cosmos-sdk and ibc-go modules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum InfractionType {
    /// UNSPECIFIED defines an empty infraction type.
    Unspecified = 0,
    /// DOUBLE_SIGN defines a validator that double-signs a block.
    DoubleSign = 1,
    /// DOWNTIME defines a validator that missed signing too many blocks.
    Downtime = 2,
}
impl InfractionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            InfractionType::Unspecified => "INFRACTION_TYPE_UNSPECIFIED",
            InfractionType::DoubleSign => "INFRACTION_TYPE_DOUBLE_SIGN",
            InfractionType::Downtime => "INFRACTION_TYPE_DOWNTIME",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "INFRACTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "INFRACTION_TYPE_DOUBLE_SIGN" => Some(Self::DoubleSign),
            "INFRACTION_TYPE_DOWNTIME" => Some(Self::Downtime),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
