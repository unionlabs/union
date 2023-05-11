/// Channel defines pipeline for exactly-once packet delivery between specific
/// modules on separate blockchains, which has at least one end capable of
/// sending packets and one end capable of receiving packets.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Channel {
    /// current state of the channel end
    #[prost(enumeration = "State", tag = "1")]
    pub state: i32,
    /// whether the channel is ordered or unordered
    #[prost(enumeration = "Order", tag = "2")]
    pub ordering: i32,
    /// counterparty channel end
    #[prost(message, optional, tag = "3")]
    pub counterparty: ::core::option::Option<Counterparty>,
    /// list of connection identifiers, in order, along which packets sent on
    /// this channel will travel
    #[prost(string, repeated, tag = "4")]
    pub connection_hops: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// opaque channel version, which is agreed upon during the handshake
    #[prost(string, tag = "5")]
    pub version: ::prost::alloc::string::String,
}
/// IdentifiedChannel defines a channel with additional port and channel
/// identifier fields.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedChannel {
    /// current state of the channel end
    #[prost(enumeration = "State", tag = "1")]
    pub state: i32,
    /// whether the channel is ordered or unordered
    #[prost(enumeration = "Order", tag = "2")]
    pub ordering: i32,
    /// counterparty channel end
    #[prost(message, optional, tag = "3")]
    pub counterparty: ::core::option::Option<Counterparty>,
    /// list of connection identifiers, in order, along which packets sent on
    /// this channel will travel
    #[prost(string, repeated, tag = "4")]
    pub connection_hops: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// opaque channel version, which is agreed upon during the handshake
    #[prost(string, tag = "5")]
    pub version: ::prost::alloc::string::String,
    /// port identifier
    #[prost(string, tag = "6")]
    pub port_id: ::prost::alloc::string::String,
    /// channel identifier
    #[prost(string, tag = "7")]
    pub channel_id: ::prost::alloc::string::String,
}
/// Counterparty defines a channel end counterparty
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Counterparty {
    /// port on the counterparty chain which owns the other end of the channel.
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel end on the counterparty chain
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// Packet defines a type that carries data across different chains through IBC
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Packet {
    /// number corresponds to the order of sends and receives, where a Packet
    /// with an earlier sequence number must be sent and received before a Packet
    /// with a later sequence number.
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    /// identifies the port on the sending chain.
    #[prost(string, tag = "2")]
    pub source_port: ::prost::alloc::string::String,
    /// identifies the channel end on the sending chain.
    #[prost(string, tag = "3")]
    pub source_channel: ::prost::alloc::string::String,
    /// identifies the port on the receiving chain.
    #[prost(string, tag = "4")]
    pub destination_port: ::prost::alloc::string::String,
    /// identifies the channel end on the receiving chain.
    #[prost(string, tag = "5")]
    pub destination_channel: ::prost::alloc::string::String,
    /// actual opaque bytes transferred directly to the application module
    #[prost(bytes = "vec", tag = "6")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// block height after which the packet times out
    #[prost(message, optional, tag = "7")]
    pub timeout_height: ::core::option::Option<super::super::client::v1::Height>,
    /// block timestamp (in nanoseconds) after which the packet times out
    #[prost(uint64, tag = "8")]
    pub timeout_timestamp: u64,
}
/// PacketState defines the generic type necessary to retrieve and store
/// packet commitments, acknowledgements, and receipts.
/// Caller is responsible for knowing the context necessary to interpret this
/// state as a commitment, acknowledgement, or a receipt.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketState {
    /// channel port identifier.
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier.
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// packet sequence.
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
    /// embedded data that represents packet state.
    #[prost(bytes = "vec", tag = "4")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// PacketId is an identifer for a unique Packet
/// Source chains refer to packets by source port/channel
/// Destination chains refer to packets by destination port/channel
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketId {
    /// channel port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// packet sequence
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// Acknowledgement is the recommended acknowledgement format to be used by
/// app-specific protocols.
/// NOTE: The field numbers 21 and 22 were explicitly chosen to avoid accidental
/// conflicts with other protobuf message formats used for acknowledgements.
/// The first byte of any message with this format will be the non-ASCII values
/// `0xaa` (result) or `0xb2` (error). Implemented as defined by ICS:
/// <https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#acknowledgement-envelope>
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Acknowledgement {
    /// response contains either a result or an error and must be non-empty
    #[prost(oneof = "acknowledgement::Response", tags = "21, 22")]
    pub response: ::core::option::Option<acknowledgement::Response>,
}
/// Nested message and enum types in `Acknowledgement`.
pub mod acknowledgement {
    /// response contains either a result or an error and must be non-empty
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(bytes, tag = "21")]
        Result(::prost::alloc::vec::Vec<u8>),
        #[prost(string, tag = "22")]
        Error(::prost::alloc::string::String),
    }
}
/// State defines if a channel is in one of the following states:
/// CLOSED, INIT, TRYOPEN, OPEN or UNINITIALIZED.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum State {
    /// Default State
    UninitializedUnspecified = 0,
    /// A channel has just started the opening handshake.
    Init = 1,
    /// A channel has acknowledged the handshake step on the counterparty chain.
    Tryopen = 2,
    /// A channel has completed the handshake. Open channels are
    /// ready to send and receive packets.
    Open = 3,
    /// A channel has been closed and can no longer be used to send or receive
    /// packets.
    Closed = 4,
}
impl State {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            State::UninitializedUnspecified => "STATE_UNINITIALIZED_UNSPECIFIED",
            State::Init => "STATE_INIT",
            State::Tryopen => "STATE_TRYOPEN",
            State::Open => "STATE_OPEN",
            State::Closed => "STATE_CLOSED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "STATE_UNINITIALIZED_UNSPECIFIED" => Some(Self::UninitializedUnspecified),
            "STATE_INIT" => Some(Self::Init),
            "STATE_TRYOPEN" => Some(Self::Tryopen),
            "STATE_OPEN" => Some(Self::Open),
            "STATE_CLOSED" => Some(Self::Closed),
            _ => None,
        }
    }
}
/// Order defines if a channel is ORDERED or UNORDERED
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Order {
    /// zero-value for channel ordering
    NoneUnspecified = 0,
    /// packets can be delivered in any order, which may differ from the order in
    /// which they were sent.
    Unordered = 1,
    /// packets are delivered exactly in the order which they were sent
    Ordered = 2,
}
impl Order {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Order::NoneUnspecified => "ORDER_NONE_UNSPECIFIED",
            Order::Unordered => "ORDER_UNORDERED",
            Order::Ordered => "ORDER_ORDERED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ORDER_NONE_UNSPECIFIED" => Some(Self::NoneUnspecified),
            "ORDER_UNORDERED" => Some(Self::Unordered),
            "ORDER_ORDERED" => Some(Self::Ordered),
            _ => None,
        }
    }
}
/// QueryChannelRequest is the request type for the Query/Channel RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryChannelResponse is the response type for the Query/Channel RPC method.
/// Besides the Channel end, it includes a proof and the height from which the
/// proof was retrieved.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelResponse {
    /// channel associated with the request identifiers
    #[prost(message, optional, tag = "1")]
    pub channel: ::core::option::Option<Channel>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryChannelsRequest is the request type for the Query/Channels RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelsRequest {
    /// pagination request
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryChannelsResponse is the response type for the Query/Channels RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelsResponse {
    /// list of stored channels of the chain.
    #[prost(message, repeated, tag = "1")]
    pub channels: ::prost::alloc::vec::Vec<IdentifiedChannel>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
    /// query block height
    #[prost(message, optional, tag = "3")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryConnectionChannelsRequest is the request type for the
/// Query/QueryConnectionChannels RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionChannelsRequest {
    /// connection unique identifier
    #[prost(string, tag = "1")]
    pub connection: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryConnectionChannelsResponse is the Response type for the
/// Query/QueryConnectionChannels RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionChannelsResponse {
    /// list of channels associated with a connection.
    #[prost(message, repeated, tag = "1")]
    pub channels: ::prost::alloc::vec::Vec<IdentifiedChannel>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
    /// query block height
    #[prost(message, optional, tag = "3")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryChannelClientStateRequest is the request type for the Query/ClientState
/// RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelClientStateRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryChannelClientStateResponse is the Response type for the
/// Query/QueryChannelClientState RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelClientStateResponse {
    /// client state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub identified_client_state: ::core::option::Option<
        super::super::client::v1::IdentifiedClientState,
    >,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryChannelConsensusStateRequest is the request type for the
/// Query/ConsensusState RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelConsensusStateRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// revision number of the consensus state
    #[prost(uint64, tag = "3")]
    pub revision_number: u64,
    /// revision height of the consensus state
    #[prost(uint64, tag = "4")]
    pub revision_height: u64,
}
/// QueryChannelClientStateResponse is the Response type for the
/// Query/QueryChannelClientState RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryChannelConsensusStateResponse {
    /// consensus state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// client ID associated with the consensus state
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryPacketCommitmentRequest is the request type for the
/// Query/PacketCommitment RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketCommitmentRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// packet sequence
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// QueryPacketCommitmentResponse defines the client query response for a packet
/// which also includes a proof and the height from which the proof was
/// retrieved
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketCommitmentResponse {
    /// packet associated with the request fields
    #[prost(bytes = "vec", tag = "1")]
    pub commitment: ::prost::alloc::vec::Vec<u8>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryPacketCommitmentsRequest is the request type for the
/// Query/QueryPacketCommitments RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketCommitmentsRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryPacketCommitmentsResponse is the request type for the
/// Query/QueryPacketCommitments RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketCommitmentsResponse {
    #[prost(message, repeated, tag = "1")]
    pub commitments: ::prost::alloc::vec::Vec<PacketState>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
    /// query block height
    #[prost(message, optional, tag = "3")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryPacketReceiptRequest is the request type for the
/// Query/PacketReceipt RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketReceiptRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// packet sequence
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// QueryPacketReceiptResponse defines the client query response for a packet
/// receipt which also includes a proof, and the height from which the proof was
/// retrieved
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketReceiptResponse {
    /// success flag for if receipt exists
    #[prost(bool, tag = "2")]
    pub received: bool,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryPacketAcknowledgementRequest is the request type for the
/// Query/PacketAcknowledgement RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketAcknowledgementRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// packet sequence
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// QueryPacketAcknowledgementResponse defines the client query response for a
/// packet which also includes a proof and the height from which the
/// proof was retrieved
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketAcknowledgementResponse {
    /// packet associated with the request fields
    #[prost(bytes = "vec", tag = "1")]
    pub acknowledgement: ::prost::alloc::vec::Vec<u8>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryPacketAcknowledgementsRequest is the request type for the
/// Query/QueryPacketCommitments RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketAcknowledgementsRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
    /// list of packet sequences
    #[prost(uint64, repeated, tag = "4")]
    pub packet_commitment_sequences: ::prost::alloc::vec::Vec<u64>,
}
/// QueryPacketAcknowledgemetsResponse is the request type for the
/// Query/QueryPacketAcknowledgements RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPacketAcknowledgementsResponse {
    #[prost(message, repeated, tag = "1")]
    pub acknowledgements: ::prost::alloc::vec::Vec<PacketState>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
    /// query block height
    #[prost(message, optional, tag = "3")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryUnreceivedPacketsRequest is the request type for the
/// Query/UnreceivedPackets RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnreceivedPacketsRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// list of packet sequences
    #[prost(uint64, repeated, tag = "3")]
    pub packet_commitment_sequences: ::prost::alloc::vec::Vec<u64>,
}
/// QueryUnreceivedPacketsResponse is the response type for the
/// Query/UnreceivedPacketCommitments RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnreceivedPacketsResponse {
    /// list of unreceived packet sequences
    #[prost(uint64, repeated, tag = "1")]
    pub sequences: ::prost::alloc::vec::Vec<u64>,
    /// query block height
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryUnreceivedAcks is the request type for the
/// Query/UnreceivedAcks RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnreceivedAcksRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// list of acknowledgement sequences
    #[prost(uint64, repeated, tag = "3")]
    pub packet_ack_sequences: ::prost::alloc::vec::Vec<u64>,
}
/// QueryUnreceivedAcksResponse is the response type for the
/// Query/UnreceivedAcks RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnreceivedAcksResponse {
    /// list of unreceived acknowledgement sequences
    #[prost(uint64, repeated, tag = "1")]
    pub sequences: ::prost::alloc::vec::Vec<u64>,
    /// query block height
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryNextSequenceReceiveRequest is the request type for the
/// Query/QueryNextSequenceReceiveRequest RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextSequenceReceiveRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QuerySequenceResponse is the request type for the
/// Query/QueryNextSequenceReceiveResponse RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextSequenceReceiveResponse {
    /// next sequence receive number
    #[prost(uint64, tag = "1")]
    pub next_sequence_receive: u64,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryNextSequenceSendRequest is the request type for the
/// Query/QueryNextSequenceSend RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextSequenceSendRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryNextSequenceSendResponse is the request type for the
/// Query/QueryNextSequenceSend RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextSequenceSendResponse {
    /// next sequence send number
    #[prost(uint64, tag = "1")]
    pub next_sequence_send: u64,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query provides defines the gRPC querier service
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Channel queries an IBC Channel.
        pub async fn channel(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/Channel",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "Channel"));
            self.inner.unary(req, path, codec).await
        }
        /// Channels queries all the IBC channels of a chain.
        pub async fn channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/Channels",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "Channels"));
            self.inner.unary(req, path, codec).await
        }
        /// ConnectionChannels queries all the channels associated with a connection
        /// end.
        pub async fn connection_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConnectionChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConnectionChannelsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ConnectionChannels",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "ConnectionChannels"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ChannelClientState queries for the client state for the channel associated
        /// with the provided channel identifiers.
        pub async fn channel_client_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelClientStateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ChannelClientState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "ChannelClientState"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ChannelConsensusState queries for the consensus state for the channel
        /// associated with the provided channel identifiers.
        pub async fn channel_consensus_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelConsensusStateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ChannelConsensusState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "ChannelConsensusState"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PacketCommitment queries a stored packet commitment hash.
        pub async fn packet_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketCommitmentResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketCommitment",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "PacketCommitment"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PacketCommitments returns all the packet commitments hashes associated
        /// with a channel.
        pub async fn packet_commitments(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketCommitmentsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketCommitmentsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketCommitments",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "PacketCommitments"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PacketReceipt queries if a given packet sequence has been received on the
        /// queried chain
        pub async fn packet_receipt(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketReceiptRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketReceiptResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketReceipt",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "PacketReceipt"));
            self.inner.unary(req, path, codec).await
        }
        /// PacketAcknowledgement queries a stored packet acknowledgement hash.
        pub async fn packet_acknowledgement(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketAcknowledgementRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketAcknowledgement",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "PacketAcknowledgement"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PacketAcknowledgements returns all the packet acknowledgements associated
        /// with a channel.
        pub async fn packet_acknowledgements(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketAcknowledgementsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketAcknowledgements",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.core.channel.v1.Query",
                        "PacketAcknowledgements",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// UnreceivedPackets returns all the unreceived IBC packets associated with a
        /// channel and sequences.
        pub async fn unreceived_packets(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnreceivedPacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnreceivedPacketsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/UnreceivedPackets",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "UnreceivedPackets"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
        /// with a channel and sequences.
        pub async fn unreceived_acks(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnreceivedAcksRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnreceivedAcksResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/UnreceivedAcks",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "UnreceivedAcks"));
            self.inner.unary(req, path, codec).await
        }
        /// NextSequenceReceive returns the next receive sequence for a given channel.
        pub async fn next_sequence_receive(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryNextSequenceReceiveRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextSequenceReceiveResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/NextSequenceReceive",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "NextSequenceReceive"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// NextSequenceSend returns the next send sequence for a given channel.
        pub async fn next_sequence_send(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryNextSequenceSendRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextSequenceSendResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/NextSequenceSend",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Query", "NextSequenceSend"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        /// Channel queries an IBC Channel.
        async fn channel(
            &self,
            request: tonic::Request<super::QueryChannelRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelResponse>,
            tonic::Status,
        >;
        /// Channels queries all the IBC channels of a chain.
        async fn channels(
            &self,
            request: tonic::Request<super::QueryChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelsResponse>,
            tonic::Status,
        >;
        /// ConnectionChannels queries all the channels associated with a connection
        /// end.
        async fn connection_channels(
            &self,
            request: tonic::Request<super::QueryConnectionChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConnectionChannelsResponse>,
            tonic::Status,
        >;
        /// ChannelClientState queries for the client state for the channel associated
        /// with the provided channel identifiers.
        async fn channel_client_state(
            &self,
            request: tonic::Request<super::QueryChannelClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelClientStateResponse>,
            tonic::Status,
        >;
        /// ChannelConsensusState queries for the consensus state for the channel
        /// associated with the provided channel identifiers.
        async fn channel_consensus_state(
            &self,
            request: tonic::Request<super::QueryChannelConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelConsensusStateResponse>,
            tonic::Status,
        >;
        /// PacketCommitment queries a stored packet commitment hash.
        async fn packet_commitment(
            &self,
            request: tonic::Request<super::QueryPacketCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketCommitmentResponse>,
            tonic::Status,
        >;
        /// PacketCommitments returns all the packet commitments hashes associated
        /// with a channel.
        async fn packet_commitments(
            &self,
            request: tonic::Request<super::QueryPacketCommitmentsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketCommitmentsResponse>,
            tonic::Status,
        >;
        /// PacketReceipt queries if a given packet sequence has been received on the
        /// queried chain
        async fn packet_receipt(
            &self,
            request: tonic::Request<super::QueryPacketReceiptRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketReceiptResponse>,
            tonic::Status,
        >;
        /// PacketAcknowledgement queries a stored packet acknowledgement hash.
        async fn packet_acknowledgement(
            &self,
            request: tonic::Request<super::QueryPacketAcknowledgementRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementResponse>,
            tonic::Status,
        >;
        /// PacketAcknowledgements returns all the packet acknowledgements associated
        /// with a channel.
        async fn packet_acknowledgements(
            &self,
            request: tonic::Request<super::QueryPacketAcknowledgementsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementsResponse>,
            tonic::Status,
        >;
        /// UnreceivedPackets returns all the unreceived IBC packets associated with a
        /// channel and sequences.
        async fn unreceived_packets(
            &self,
            request: tonic::Request<super::QueryUnreceivedPacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnreceivedPacketsResponse>,
            tonic::Status,
        >;
        /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
        /// with a channel and sequences.
        async fn unreceived_acks(
            &self,
            request: tonic::Request<super::QueryUnreceivedAcksRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnreceivedAcksResponse>,
            tonic::Status,
        >;
        /// NextSequenceReceive returns the next receive sequence for a given channel.
        async fn next_sequence_receive(
            &self,
            request: tonic::Request<super::QueryNextSequenceReceiveRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextSequenceReceiveResponse>,
            tonic::Status,
        >;
        /// NextSequenceSend returns the next send sequence for a given channel.
        async fn next_sequence_send(
            &self,
            request: tonic::Request<super::QueryNextSequenceSendRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextSequenceSendResponse>,
            tonic::Status,
        >;
    }
    /// Query provides defines the gRPC querier service
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ibc.core.channel.v1.Query/Channel" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryChannelRequest>
                    for ChannelSvc<T> {
                        type Response = super::QueryChannelResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryChannelRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).channel(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/Channels" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryChannelsRequest>
                    for ChannelsSvc<T> {
                        type Response = super::QueryChannelsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryChannelsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).channels(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/ConnectionChannels" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectionChannelsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryConnectionChannelsRequest>
                    for ConnectionChannelsSvc<T> {
                        type Response = super::QueryConnectionChannelsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryConnectionChannelsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).connection_channels(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectionChannelsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/ChannelClientState" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelClientStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryChannelClientStateRequest>
                    for ChannelClientStateSvc<T> {
                        type Response = super::QueryChannelClientStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryChannelClientStateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_client_state(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelClientStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/ChannelConsensusState" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelConsensusStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryChannelConsensusStateRequest,
                    > for ChannelConsensusStateSvc<T> {
                        type Response = super::QueryChannelConsensusStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryChannelConsensusStateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_consensus_state(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelConsensusStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/PacketCommitment" => {
                    #[allow(non_camel_case_types)]
                    struct PacketCommitmentSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryPacketCommitmentRequest>
                    for PacketCommitmentSvc<T> {
                        type Response = super::QueryPacketCommitmentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryPacketCommitmentRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).packet_commitment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PacketCommitmentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/PacketCommitments" => {
                    #[allow(non_camel_case_types)]
                    struct PacketCommitmentsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryPacketCommitmentsRequest>
                    for PacketCommitmentsSvc<T> {
                        type Response = super::QueryPacketCommitmentsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryPacketCommitmentsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).packet_commitments(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PacketCommitmentsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/PacketReceipt" => {
                    #[allow(non_camel_case_types)]
                    struct PacketReceiptSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryPacketReceiptRequest>
                    for PacketReceiptSvc<T> {
                        type Response = super::QueryPacketReceiptResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryPacketReceiptRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).packet_receipt(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PacketReceiptSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/PacketAcknowledgement" => {
                    #[allow(non_camel_case_types)]
                    struct PacketAcknowledgementSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryPacketAcknowledgementRequest,
                    > for PacketAcknowledgementSvc<T> {
                        type Response = super::QueryPacketAcknowledgementResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryPacketAcknowledgementRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).packet_acknowledgement(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PacketAcknowledgementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/PacketAcknowledgements" => {
                    #[allow(non_camel_case_types)]
                    struct PacketAcknowledgementsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryPacketAcknowledgementsRequest,
                    > for PacketAcknowledgementsSvc<T> {
                        type Response = super::QueryPacketAcknowledgementsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryPacketAcknowledgementsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).packet_acknowledgements(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PacketAcknowledgementsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/UnreceivedPackets" => {
                    #[allow(non_camel_case_types)]
                    struct UnreceivedPacketsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryUnreceivedPacketsRequest>
                    for UnreceivedPacketsSvc<T> {
                        type Response = super::QueryUnreceivedPacketsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryUnreceivedPacketsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).unreceived_packets(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UnreceivedPacketsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/UnreceivedAcks" => {
                    #[allow(non_camel_case_types)]
                    struct UnreceivedAcksSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryUnreceivedAcksRequest>
                    for UnreceivedAcksSvc<T> {
                        type Response = super::QueryUnreceivedAcksResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryUnreceivedAcksRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).unreceived_acks(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UnreceivedAcksSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/NextSequenceReceive" => {
                    #[allow(non_camel_case_types)]
                    struct NextSequenceReceiveSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryNextSequenceReceiveRequest>
                    for NextSequenceReceiveSvc<T> {
                        type Response = super::QueryNextSequenceReceiveResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryNextSequenceReceiveRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).next_sequence_receive(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NextSequenceReceiveSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Query/NextSequenceSend" => {
                    #[allow(non_camel_case_types)]
                    struct NextSequenceSendSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryNextSequenceSendRequest>
                    for NextSequenceSendSvc<T> {
                        type Response = super::QueryNextSequenceSendResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryNextSequenceSendRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).next_sequence_send(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NextSequenceSendSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Query> Clone for QueryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "ibc.core.channel.v1.Query";
    }
}
/// MsgChannelOpenInit defines an sdk.Msg to initialize a channel handshake. It
/// is called by a relayer on Chain A.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenInit {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub channel: ::core::option::Option<Channel>,
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelOpenInitResponse defines the Msg/ChannelOpenInit response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenInitResponse {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
}
/// MsgChannelOpenInit defines a msg sent by a Relayer to try to open a channel
/// on Chain B. The version field within the Channel field has been deprecated. Its
/// value will be ignored by core IBC.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenTry {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// Deprecated: this field is unused. Crossing hello's are no longer supported in core IBC.
    #[deprecated]
    #[prost(string, tag = "2")]
    pub previous_channel_id: ::prost::alloc::string::String,
    /// NOTE: the version field within the channel has been deprecated. Its value will be ignored by core IBC.
    #[prost(message, optional, tag = "3")]
    pub channel: ::core::option::Option<Channel>,
    #[prost(string, tag = "4")]
    pub counterparty_version: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "5")]
    pub proof_init: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "7")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelOpenTryResponse defines the Msg/ChannelOpenTry response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenTryResponse {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// MsgChannelOpenAck defines a msg sent by a Relayer to Chain A to acknowledge
/// the change of channel state to TRYOPEN on Chain B.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenAck {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub counterparty_channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub counterparty_version: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "5")]
    pub proof_try: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "7")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelOpenAckResponse defines the Msg/ChannelOpenAck response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenAckResponse {}
/// MsgChannelOpenConfirm defines a msg sent by a Relayer to Chain B to
/// acknowledge the change of channel state to OPEN on Chain A.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenConfirm {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub proof_ack: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "5")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelOpenConfirmResponse defines the Msg/ChannelOpenConfirm response
/// type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelOpenConfirmResponse {}
/// MsgChannelCloseInit defines a msg sent by a Relayer to Chain A
/// to close a channel with Chain B.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelCloseInit {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelCloseInitResponse defines the Msg/ChannelCloseInit response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelCloseInitResponse {}
/// MsgChannelCloseConfirm defines a msg sent by a Relayer to Chain B
/// to acknowledge the change of channel state to CLOSED on Chain A.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelCloseConfirm {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub proof_init: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "5")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelCloseConfirmResponse defines the Msg/ChannelCloseConfirm response
/// type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChannelCloseConfirmResponse {}
/// MsgRecvPacket receives incoming IBC packet
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRecvPacket {
    #[prost(message, optional, tag = "1")]
    pub packet: ::core::option::Option<Packet>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof_commitment: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgRecvPacketResponse defines the Msg/RecvPacket response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRecvPacketResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgTimeout receives timed-out packet
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTimeout {
    #[prost(message, optional, tag = "1")]
    pub packet: ::core::option::Option<Packet>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof_unreceived: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(uint64, tag = "4")]
    pub next_sequence_recv: u64,
    #[prost(string, tag = "5")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgTimeoutResponse defines the Msg/Timeout response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTimeoutResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgTimeoutOnClose timed-out packet upon counterparty channel closure.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTimeoutOnClose {
    #[prost(message, optional, tag = "1")]
    pub packet: ::core::option::Option<Packet>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof_unreceived: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub proof_close: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(uint64, tag = "5")]
    pub next_sequence_recv: u64,
    #[prost(string, tag = "6")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgTimeoutOnCloseResponse defines the Msg/TimeoutOnClose response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTimeoutOnCloseResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgAcknowledgement receives incoming IBC acknowledgement
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAcknowledgement {
    #[prost(message, optional, tag = "1")]
    pub packet: ::core::option::Option<Packet>,
    #[prost(bytes = "vec", tag = "2")]
    pub acknowledgement: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub proof_acked: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "5")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgAcknowledgementResponse defines the Msg/Acknowledgement response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAcknowledgementResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// ResponseResultType defines the possible outcomes of the execution of a message
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResponseResultType {
    /// Default zero value enumeration
    Unspecified = 0,
    /// The message did not call the IBC application callbacks (because, for example, the packet had already been relayed)
    Noop = 1,
    /// The message was executed successfully
    Success = 2,
}
impl ResponseResultType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ResponseResultType::Unspecified => "RESPONSE_RESULT_TYPE_UNSPECIFIED",
            ResponseResultType::Noop => "RESPONSE_RESULT_TYPE_NOOP",
            ResponseResultType::Success => "RESPONSE_RESULT_TYPE_SUCCESS",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RESPONSE_RESULT_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "RESPONSE_RESULT_TYPE_NOOP" => Some(Self::Noop),
            "RESPONSE_RESULT_TYPE_SUCCESS" => Some(Self::Success),
            _ => None,
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ibc/channel Msg service.
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsgClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MsgClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// ChannelOpenInit defines a rpc handler method for MsgChannelOpenInit.
        pub async fn channel_open_init(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenInit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenInitResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelOpenInit",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenInit"));
            self.inner.unary(req, path, codec).await
        }
        /// ChannelOpenTry defines a rpc handler method for MsgChannelOpenTry.
        pub async fn channel_open_try(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenTry>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenTryResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelOpenTry",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenTry"));
            self.inner.unary(req, path, codec).await
        }
        /// ChannelOpenAck defines a rpc handler method for MsgChannelOpenAck.
        pub async fn channel_open_ack(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenAck>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenAckResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelOpenAck",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenAck"));
            self.inner.unary(req, path, codec).await
        }
        /// ChannelOpenConfirm defines a rpc handler method for MsgChannelOpenConfirm.
        pub async fn channel_open_confirm(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenConfirmResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelOpenConfirm",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenConfirm"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ChannelCloseInit defines a rpc handler method for MsgChannelCloseInit.
        pub async fn channel_close_init(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelCloseInit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelCloseInitResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelCloseInit",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelCloseInit"));
            self.inner.unary(req, path, codec).await
        }
        /// ChannelCloseConfirm defines a rpc handler method for
        /// MsgChannelCloseConfirm.
        pub async fn channel_close_confirm(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelCloseConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelCloseConfirmResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelCloseConfirm",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelCloseConfirm"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// RecvPacket defines a rpc handler method for MsgRecvPacket.
        pub async fn recv_packet(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRecvPacket>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRecvPacketResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/RecvPacket",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "RecvPacket"));
            self.inner.unary(req, path, codec).await
        }
        /// Timeout defines a rpc handler method for MsgTimeout.
        pub async fn timeout(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgTimeout>,
        ) -> std::result::Result<
            tonic::Response<super::MsgTimeoutResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/Timeout",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "Timeout"));
            self.inner.unary(req, path, codec).await
        }
        /// TimeoutOnClose defines a rpc handler method for MsgTimeoutOnClose.
        pub async fn timeout_on_close(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgTimeoutOnClose>,
        ) -> std::result::Result<
            tonic::Response<super::MsgTimeoutOnCloseResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/TimeoutOnClose",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "TimeoutOnClose"));
            self.inner.unary(req, path, codec).await
        }
        /// Acknowledgement defines a rpc handler method for MsgAcknowledgement.
        pub async fn acknowledgement(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAcknowledgement>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAcknowledgementResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/Acknowledgement",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "Acknowledgement"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        /// ChannelOpenInit defines a rpc handler method for MsgChannelOpenInit.
        async fn channel_open_init(
            &self,
            request: tonic::Request<super::MsgChannelOpenInit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenInitResponse>,
            tonic::Status,
        >;
        /// ChannelOpenTry defines a rpc handler method for MsgChannelOpenTry.
        async fn channel_open_try(
            &self,
            request: tonic::Request<super::MsgChannelOpenTry>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenTryResponse>,
            tonic::Status,
        >;
        /// ChannelOpenAck defines a rpc handler method for MsgChannelOpenAck.
        async fn channel_open_ack(
            &self,
            request: tonic::Request<super::MsgChannelOpenAck>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenAckResponse>,
            tonic::Status,
        >;
        /// ChannelOpenConfirm defines a rpc handler method for MsgChannelOpenConfirm.
        async fn channel_open_confirm(
            &self,
            request: tonic::Request<super::MsgChannelOpenConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelOpenConfirmResponse>,
            tonic::Status,
        >;
        /// ChannelCloseInit defines a rpc handler method for MsgChannelCloseInit.
        async fn channel_close_init(
            &self,
            request: tonic::Request<super::MsgChannelCloseInit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelCloseInitResponse>,
            tonic::Status,
        >;
        /// ChannelCloseConfirm defines a rpc handler method for
        /// MsgChannelCloseConfirm.
        async fn channel_close_confirm(
            &self,
            request: tonic::Request<super::MsgChannelCloseConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelCloseConfirmResponse>,
            tonic::Status,
        >;
        /// RecvPacket defines a rpc handler method for MsgRecvPacket.
        async fn recv_packet(
            &self,
            request: tonic::Request<super::MsgRecvPacket>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRecvPacketResponse>,
            tonic::Status,
        >;
        /// Timeout defines a rpc handler method for MsgTimeout.
        async fn timeout(
            &self,
            request: tonic::Request<super::MsgTimeout>,
        ) -> std::result::Result<
            tonic::Response<super::MsgTimeoutResponse>,
            tonic::Status,
        >;
        /// TimeoutOnClose defines a rpc handler method for MsgTimeoutOnClose.
        async fn timeout_on_close(
            &self,
            request: tonic::Request<super::MsgTimeoutOnClose>,
        ) -> std::result::Result<
            tonic::Response<super::MsgTimeoutOnCloseResponse>,
            tonic::Status,
        >;
        /// Acknowledgement defines a rpc handler method for MsgAcknowledgement.
        async fn acknowledgement(
            &self,
            request: tonic::Request<super::MsgAcknowledgement>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAcknowledgementResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the ibc/channel Msg service.
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ibc.core.channel.v1.Msg/ChannelOpenInit" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenInitSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgChannelOpenInit>
                    for ChannelOpenInitSvc<T> {
                        type Response = super::MsgChannelOpenInitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelOpenInit>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_open_init(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenInitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/ChannelOpenTry" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenTrySvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgChannelOpenTry>
                    for ChannelOpenTrySvc<T> {
                        type Response = super::MsgChannelOpenTryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelOpenTry>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_open_try(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenTrySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/ChannelOpenAck" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenAckSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgChannelOpenAck>
                    for ChannelOpenAckSvc<T> {
                        type Response = super::MsgChannelOpenAckResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelOpenAck>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_open_ack(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenAckSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/ChannelOpenConfirm" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenConfirmSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<super::MsgChannelOpenConfirm>
                    for ChannelOpenConfirmSvc<T> {
                        type Response = super::MsgChannelOpenConfirmResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelOpenConfirm>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_open_confirm(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenConfirmSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/ChannelCloseInit" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelCloseInitSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgChannelCloseInit>
                    for ChannelCloseInitSvc<T> {
                        type Response = super::MsgChannelCloseInitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelCloseInit>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_close_init(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelCloseInitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/ChannelCloseConfirm" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelCloseConfirmSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<super::MsgChannelCloseConfirm>
                    for ChannelCloseConfirmSvc<T> {
                        type Response = super::MsgChannelCloseConfirmResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgChannelCloseConfirm>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).channel_close_confirm(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelCloseConfirmSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/RecvPacket" => {
                    #[allow(non_camel_case_types)]
                    struct RecvPacketSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgRecvPacket>
                    for RecvPacketSvc<T> {
                        type Response = super::MsgRecvPacketResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgRecvPacket>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).recv_packet(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RecvPacketSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/Timeout" => {
                    #[allow(non_camel_case_types)]
                    struct TimeoutSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgTimeout>
                    for TimeoutSvc<T> {
                        type Response = super::MsgTimeoutResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgTimeout>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).timeout(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TimeoutSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/TimeoutOnClose" => {
                    #[allow(non_camel_case_types)]
                    struct TimeoutOnCloseSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgTimeoutOnClose>
                    for TimeoutOnCloseSvc<T> {
                        type Response = super::MsgTimeoutOnCloseResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgTimeoutOnClose>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).timeout_on_close(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TimeoutOnCloseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.core.channel.v1.Msg/Acknowledgement" => {
                    #[allow(non_camel_case_types)]
                    struct AcknowledgementSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgAcknowledgement>
                    for AcknowledgementSvc<T> {
                        type Response = super::MsgAcknowledgementResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgAcknowledgement>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).acknowledgement(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AcknowledgementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Msg> Clone for MsgServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "ibc.core.channel.v1.Msg";
    }
}
/// GenesisState defines the ibc channel submodule's genesis state.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub channels: ::prost::alloc::vec::Vec<IdentifiedChannel>,
    #[prost(message, repeated, tag = "2")]
    pub acknowledgements: ::prost::alloc::vec::Vec<PacketState>,
    #[prost(message, repeated, tag = "3")]
    pub commitments: ::prost::alloc::vec::Vec<PacketState>,
    #[prost(message, repeated, tag = "4")]
    pub receipts: ::prost::alloc::vec::Vec<PacketState>,
    #[prost(message, repeated, tag = "5")]
    pub send_sequences: ::prost::alloc::vec::Vec<PacketSequence>,
    #[prost(message, repeated, tag = "6")]
    pub recv_sequences: ::prost::alloc::vec::Vec<PacketSequence>,
    #[prost(message, repeated, tag = "7")]
    pub ack_sequences: ::prost::alloc::vec::Vec<PacketSequence>,
    /// the sequence for the next generated channel identifier
    #[prost(uint64, tag = "8")]
    pub next_channel_sequence: u64,
}
/// PacketSequence defines the genesis type necessary to retrieve and store
/// next send and receive sequences.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketSequence {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
