/// Acknowledgement is the recommended acknowledgement format to be used by
/// app-specific protocols.
/// NOTE: The field numbers 21 and 22 were explicitly chosen to avoid accidental
/// conflicts with other protobuf message formats used for acknowledgements.
/// The first byte of any message with this format will be the non-ASCII values
/// `0xaa` (result) or `0xb2` (error). Implemented as defined by ICS:
/// <https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#acknowledgement-envelope>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Acknowledgement {
    /// response contains either a result or an error and must be non-empty
    #[prost(oneof = "acknowledgement::Response", tags = "21, 22")]
    pub response: ::core::option::Option<acknowledgement::Response>,
}
/// Channel defines pipeline for exactly-once packet delivery between specific
/// modules on separate blockchains, which has at least one end capable of
/// sending packets and one end capable of receiving packets.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    /// upgrade sequence indicates the latest upgrade attempt performed by this channel
    /// the value of 0 indicates the channel has never been upgraded
    #[prost(uint64, tag = "6")]
    pub upgrade_sequence: u64,
}
/// Counterparty defines a channel end counterparty
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Counterparty {
    /// port on the counterparty chain which owns the other end of the channel.
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel end on the counterparty chain
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// ErrorReceipt defines a type which encapsulates the upgrade sequence and error associated with the
/// upgrade handshake failure. When a channel upgrade handshake is aborted both chains are expected to increment to the
/// next sequence.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ErrorReceipt {
    /// the channel upgrade sequence
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    /// the error message detailing the cause of failure
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
/// GenesisState defines the ibc channel submodule's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    #[prost(message, optional, tag = "9")]
    pub params: ::core::option::Option<Params>,
}
/// IdentifiedChannel defines a channel with additional port and channel
/// identifier fields.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    /// upgrade sequence indicates the latest upgrade attempt performed by this channel
    /// the value of 0 indicates the channel has never been upgraded
    #[prost(uint64, tag = "8")]
    pub upgrade_sequence: u64,
}
/// MsgAcknowledgement receives incoming IBC acknowledgement
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgAcknowledgementResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgChannelCloseConfirm defines a msg sent by a Relayer to Chain B
/// to acknowledge the change of channel state to CLOSED on Chain A.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    #[prost(uint64, tag = "6")]
    pub counterparty_upgrade_sequence: u64,
}
/// MsgChannelCloseConfirmResponse defines the Msg/ChannelCloseConfirm response
/// type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelCloseConfirmResponse {}
/// MsgChannelCloseInit defines a msg sent by a Relayer to Chain A
/// to close a channel with Chain B.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelCloseInit {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelCloseInitResponse defines the Msg/ChannelCloseInit response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelCloseInitResponse {}
/// MsgChannelOpenAck defines a msg sent by a Relayer to Chain A to acknowledge
/// the change of channel state to TRYOPEN on Chain B.
/// WARNING: a channel upgrade MUST NOT initialize an upgrade for this channel
/// in the same block as executing this message otherwise the counterparty will
/// be incapable of opening.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelOpenAckResponse {}
/// MsgChannelOpenConfirm defines a msg sent by a Relayer to Chain B to
/// acknowledge the change of channel state to OPEN on Chain A.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelOpenConfirmResponse {}
/// MsgChannelOpenInit defines a msg sent by a Relayer to try to open a channel
/// on Chain B. The version field within the Channel field has been deprecated. Its
/// value will be ignored by core IBC.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// MsgChannelOpenInit defines an sdk.Msg to initialize a channel handshake. It
/// is called by a relayer on Chain A.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelOpenInit {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub channel: ::core::option::Option<Channel>,
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelOpenInitResponse defines the Msg/ChannelOpenInit response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelOpenInitResponse {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
}
/// MsgChannelOpenTryResponse defines the Msg/ChannelOpenTry response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelOpenTryResponse {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeAck defines the request type for the ChannelUpgradeAck rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeAck {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub counterparty_upgrade: ::core::option::Option<Upgrade>,
    #[prost(bytes = "vec", tag = "4")]
    pub proof_channel: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub proof_upgrade: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "7")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeAckResponse defines MsgChannelUpgradeAck response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeAckResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgChannelUpgradeCancel defines the request type for the ChannelUpgradeCancel rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeCancel {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub error_receipt: ::core::option::Option<ErrorReceipt>,
    #[prost(bytes = "vec", tag = "4")]
    pub proof_error_receipt: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "6")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeCancelResponse defines the MsgChannelUpgradeCancel response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeCancelResponse {}
/// MsgChannelUpgradeConfirm defines the request type for the ChannelUpgradeConfirm rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeConfirm {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(enumeration = "State", tag = "3")]
    pub counterparty_channel_state: i32,
    #[prost(message, optional, tag = "4")]
    pub counterparty_upgrade: ::core::option::Option<Upgrade>,
    #[prost(bytes = "vec", tag = "5")]
    pub proof_channel: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub proof_upgrade: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "7")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "8")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeConfirmResponse defines MsgChannelUpgradeConfirm response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeConfirmResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgChannelUpgradeInit defines the request type for the ChannelUpgradeInit rpc
/// WARNING: Initializing a channel upgrade in the same block as opening the channel
/// may result in the counterparty being incapable of opening.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeInit {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub fields: ::core::option::Option<UpgradeFields>,
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeInitResponse defines the MsgChannelUpgradeInit response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeInitResponse {
    #[prost(message, optional, tag = "1")]
    pub upgrade: ::core::option::Option<Upgrade>,
    #[prost(uint64, tag = "2")]
    pub upgrade_sequence: u64,
}
/// MsgChannelUpgradeOpen defines the request type for the ChannelUpgradeOpen rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeOpen {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(enumeration = "State", tag = "3")]
    pub counterparty_channel_state: i32,
    #[prost(uint64, tag = "4")]
    pub counterparty_upgrade_sequence: u64,
    #[prost(bytes = "vec", tag = "5")]
    pub proof_channel: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "7")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeOpenResponse defines the MsgChannelUpgradeOpen response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeOpenResponse {}
/// MsgChannelUpgradeTimeout defines the request type for the ChannelUpgradeTimeout rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeTimeout {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub counterparty_channel: ::core::option::Option<Channel>,
    #[prost(bytes = "vec", tag = "4")]
    pub proof_channel: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "6")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeTimeoutRepsonse defines the MsgChannelUpgradeTimeout response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeTimeoutResponse {}
/// MsgChannelUpgradeTry defines the request type for the ChannelUpgradeTry rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeTry {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub proposed_upgrade_connection_hops: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub counterparty_upgrade_fields: ::core::option::Option<UpgradeFields>,
    #[prost(uint64, tag = "5")]
    pub counterparty_upgrade_sequence: u64,
    #[prost(bytes = "vec", tag = "6")]
    pub proof_channel: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub proof_upgrade: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "8")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "9")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgChannelUpgradeTryResponse defines the MsgChannelUpgradeTry response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgChannelUpgradeTryResponse {
    #[prost(message, optional, tag = "1")]
    pub upgrade: ::core::option::Option<Upgrade>,
    #[prost(uint64, tag = "2")]
    pub upgrade_sequence: u64,
    #[prost(enumeration = "ResponseResultType", tag = "3")]
    pub result: i32,
}
/// MsgPruneAcknowledgements defines the request type for the PruneAcknowledgements rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgPruneAcknowledgements {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub limit: u64,
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgPruneAcknowledgementsResponse defines the response type for the PruneAcknowledgements rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgPruneAcknowledgementsResponse {
    /// Number of sequences pruned (includes both packet acknowledgements and packet receipts where appropriate).
    #[prost(uint64, tag = "1")]
    pub total_pruned_sequences: u64,
    /// Number of sequences left after pruning.
    #[prost(uint64, tag = "2")]
    pub total_remaining_sequences: u64,
}
/// MsgRecvPacket receives incoming IBC packet
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgRecvPacketResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgTimeout receives timed-out packet
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// MsgTimeoutOnClose timed-out packet upon counterparty channel closure.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    #[prost(uint64, tag = "7")]
    pub counterparty_upgrade_sequence: u64,
}
/// MsgTimeoutOnCloseResponse defines the Msg/TimeoutOnClose response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgTimeoutOnCloseResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgTimeoutResponse defines the Msg/Timeout response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgTimeoutResponse {
    #[prost(enumeration = "ResponseResultType", tag = "1")]
    pub result: i32,
}
/// MsgUpdateParams is the MsgUpdateParams request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the channel parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
/// MsgUpdateParamsResponse defines the MsgUpdateParams response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParamsResponse {}
/// Nested message and enum types in `Acknowledgement`.
pub mod acknowledgement {
    /// response contains either a result or an error and must be non-empty
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, :: prost :: Oneof)]
    pub enum Response {
        #[prost(bytes, tag = "21")]
        Result(::prost::alloc::vec::Vec<u8>),
        #[prost(string, tag = "22")]
        Error(::prost::alloc::string::String),
    }
}
/// Order defines if a channel is ORDERED or UNORDERED
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
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
/// Packet defines a type that carries data across different chains through IBC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// PacketId is an identifer for a unique Packet
/// Source chains refer to packets by source port/channel
/// Destination chains refer to packets by destination port/channel
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// PacketSequence defines the genesis type necessary to retrieve and store
/// next send and receive sequences.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct PacketSequence {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// PacketState defines the generic type necessary to retrieve and store
/// packet commitments, acknowledgements, and receipts.
/// Caller is responsible for knowing the context necessary to interpret this
/// state as a commitment, acknowledgement, or a receipt.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// Params defines the set of IBC channel parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Params {
    /// the relative timeout after which channel upgrades will time out.
    #[prost(message, optional, tag = "1")]
    pub upgrade_timeout: ::core::option::Option<Timeout>,
}
/// QueryChannelClientStateRequest is the request type for the Query/ClientState
/// RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryChannelClientStateResponse {
    /// client state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub identified_client_state:
        ::core::option::Option<super::super::client::v1::IdentifiedClientState>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryChannelClientStateResponse is the Response type for the
/// Query/QueryChannelClientState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryChannelConsensusStateResponse {
    /// consensus state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
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
/// QueryChannelConsensusStateRequest is the request type for the
/// Query/ConsensusState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryChannelParamsRequest is the request type for the Query/ChannelParams RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryChannelParamsRequest {}
/// QueryChannelParamsResponse is the response type for the Query/ChannelParams RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryChannelParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// QueryChannelRequest is the request type for the Query/Channel RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryChannelsRequest {
    /// pagination request
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryChannelsResponse is the response type for the Query/Channels RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryNextSequenceReceiveRequest is the request type for the
/// Query/QueryNextSequenceReceiveRequest RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryNextSequenceReceiveRequest {
    /// port unique identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// channel unique identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryNextSequenceSendRequest is the request type for the
/// Query/QueryNextSequenceSend RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryPacketAcknowledgementRequest is the request type for the
/// Query/PacketAcknowledgement RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryPacketCommitmentRequest is the request type for the
/// Query/PacketCommitment RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QuerySequenceResponse is the response type for the
/// Query/QueryNextSequenceReceiveResponse RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryUnreceivedAcks is the request type for the
/// Query/UnreceivedAcks RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUnreceivedAcksResponse {
    /// list of unreceived acknowledgement sequences
    #[prost(uint64, repeated, tag = "1")]
    pub sequences: ::prost::alloc::vec::Vec<u64>,
    /// query block height
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryUnreceivedPacketsRequest is the request type for the
/// Query/UnreceivedPackets RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUnreceivedPacketsResponse {
    /// list of unreceived packet sequences
    #[prost(uint64, repeated, tag = "1")]
    pub sequences: ::prost::alloc::vec::Vec<u64>,
    /// query block height
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryUpgradeErrorRequest is the request type for the Query/QueryUpgradeError RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUpgradeErrorRequest {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryUpgradeErrorResponse is the response type for the Query/QueryUpgradeError RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUpgradeErrorResponse {
    #[prost(message, optional, tag = "1")]
    pub error_receipt: ::core::option::Option<ErrorReceipt>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// QueryUpgradeRequest is the request type for the QueryUpgradeRequest RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUpgradeRequest {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
/// QueryUpgradeResponse is the response type for the QueryUpgradeResponse RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryUpgradeResponse {
    #[prost(message, optional, tag = "1")]
    pub upgrade: ::core::option::Option<Upgrade>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
/// ResponseResultType defines the possible outcomes of the execution of a message
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
#[repr(i32)]
pub enum ResponseResultType {
    /// Default zero value enumeration
    Unspecified = 0,
    /// The message did not call the IBC application callbacks (because, for example, the packet had already been relayed)
    Noop = 1,
    /// The message was executed successfully
    Success = 2,
    /// The message was executed unsuccessfully
    Failure = 3,
}
/// State defines if a channel is in one of the following states:
/// CLOSED, INIT, TRYOPEN, OPEN, FLUSHING, FLUSHCOMPLETE or UNINITIALIZED.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
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
    /// A channel has just accepted the upgrade handshake attempt and is flushing in-flight packets.
    Flushing = 5,
    /// A channel has just completed flushing any in-flight packets.
    Flushcomplete = 6,
}
/// Timeout defines an execution deadline structure for 04-channel handlers.
/// This includes packet lifecycle handlers as well as the upgrade handshake handlers.
/// A valid Timeout contains either one or both of a timestamp and block height (sequence).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Timeout {
    /// block height after which the packet or upgrade times out
    #[prost(message, optional, tag = "1")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
    /// block timestamp (in nanoseconds) after which the packet or upgrade times out
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
/// Upgrade is a verifiable type which contains the relevant information
/// for an attempted upgrade. It provides the proposed changes to the channel
/// end, the timeout for this upgrade attempt and the next packet sequence
/// which allows the counterparty to efficiently know the highest sequence it has received.
/// The next sequence send is used for pruning and upgrading from unordered to ordered channels.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Upgrade {
    #[prost(message, optional, tag = "1")]
    pub fields: ::core::option::Option<UpgradeFields>,
    #[prost(message, optional, tag = "2")]
    pub timeout: ::core::option::Option<Timeout>,
    #[prost(uint64, tag = "3")]
    pub next_sequence_send: u64,
}
/// UpgradeFields are the fields in a channel end which may be changed
/// during a channel upgrade.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct UpgradeFields {
    #[prost(enumeration = "Order", tag = "1")]
    pub ordering: i32,
    #[prost(string, repeated, tag = "2")]
    pub connection_hops: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "3")]
    pub version: ::prost::alloc::string::String,
}
impl ::prost::Name for Acknowledgement {
    const NAME: &'static str = "Acknowledgement";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Channel {
    const NAME: &'static str = "Channel";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Counterparty {
    const NAME: &'static str = "Counterparty";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for ErrorReceipt {
    const NAME: &'static str = "ErrorReceipt";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for IdentifiedChannel {
    const NAME: &'static str = "IdentifiedChannel";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgAcknowledgement {
    const NAME: &'static str = "MsgAcknowledgement";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgAcknowledgementResponse {
    const NAME: &'static str = "MsgAcknowledgementResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelCloseConfirm {
    const NAME: &'static str = "MsgChannelCloseConfirm";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelCloseConfirmResponse {
    const NAME: &'static str = "MsgChannelCloseConfirmResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelCloseInit {
    const NAME: &'static str = "MsgChannelCloseInit";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelCloseInitResponse {
    const NAME: &'static str = "MsgChannelCloseInitResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenAck {
    const NAME: &'static str = "MsgChannelOpenAck";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenAckResponse {
    const NAME: &'static str = "MsgChannelOpenAckResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenConfirm {
    const NAME: &'static str = "MsgChannelOpenConfirm";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenConfirmResponse {
    const NAME: &'static str = "MsgChannelOpenConfirmResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenInit {
    const NAME: &'static str = "MsgChannelOpenInit";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenInitResponse {
    const NAME: &'static str = "MsgChannelOpenInitResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenTry {
    const NAME: &'static str = "MsgChannelOpenTry";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelOpenTryResponse {
    const NAME: &'static str = "MsgChannelOpenTryResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeAck {
    const NAME: &'static str = "MsgChannelUpgradeAck";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeAckResponse {
    const NAME: &'static str = "MsgChannelUpgradeAckResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeCancel {
    const NAME: &'static str = "MsgChannelUpgradeCancel";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeCancelResponse {
    const NAME: &'static str = "MsgChannelUpgradeCancelResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeConfirm {
    const NAME: &'static str = "MsgChannelUpgradeConfirm";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeConfirmResponse {
    const NAME: &'static str = "MsgChannelUpgradeConfirmResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeInit {
    const NAME: &'static str = "MsgChannelUpgradeInit";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeInitResponse {
    const NAME: &'static str = "MsgChannelUpgradeInitResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeOpen {
    const NAME: &'static str = "MsgChannelUpgradeOpen";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeOpenResponse {
    const NAME: &'static str = "MsgChannelUpgradeOpenResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeTimeout {
    const NAME: &'static str = "MsgChannelUpgradeTimeout";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeTimeoutResponse {
    const NAME: &'static str = "MsgChannelUpgradeTimeoutResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeTry {
    const NAME: &'static str = "MsgChannelUpgradeTry";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgChannelUpgradeTryResponse {
    const NAME: &'static str = "MsgChannelUpgradeTryResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgPruneAcknowledgements {
    const NAME: &'static str = "MsgPruneAcknowledgements";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgPruneAcknowledgementsResponse {
    const NAME: &'static str = "MsgPruneAcknowledgementsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgRecvPacket {
    const NAME: &'static str = "MsgRecvPacket";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgRecvPacketResponse {
    const NAME: &'static str = "MsgRecvPacketResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgTimeout {
    const NAME: &'static str = "MsgTimeout";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgTimeoutOnClose {
    const NAME: &'static str = "MsgTimeoutOnClose";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgTimeoutOnCloseResponse {
    const NAME: &'static str = "MsgTimeoutOnCloseResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgTimeoutResponse {
    const NAME: &'static str = "MsgTimeoutResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Packet {
    const NAME: &'static str = "Packet";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for PacketId {
    const NAME: &'static str = "PacketId";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for PacketSequence {
    const NAME: &'static str = "PacketSequence";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for PacketState {
    const NAME: &'static str = "PacketState";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelClientStateRequest {
    const NAME: &'static str = "QueryChannelClientStateRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelClientStateResponse {
    const NAME: &'static str = "QueryChannelClientStateResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelConsensusStateRequest {
    const NAME: &'static str = "QueryChannelConsensusStateRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelConsensusStateResponse {
    const NAME: &'static str = "QueryChannelConsensusStateResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelParamsRequest {
    const NAME: &'static str = "QueryChannelParamsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelParamsResponse {
    const NAME: &'static str = "QueryChannelParamsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelRequest {
    const NAME: &'static str = "QueryChannelRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelResponse {
    const NAME: &'static str = "QueryChannelResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelsRequest {
    const NAME: &'static str = "QueryChannelsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryChannelsResponse {
    const NAME: &'static str = "QueryChannelsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConnectionChannelsRequest {
    const NAME: &'static str = "QueryConnectionChannelsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConnectionChannelsResponse {
    const NAME: &'static str = "QueryConnectionChannelsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryNextSequenceReceiveRequest {
    const NAME: &'static str = "QueryNextSequenceReceiveRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryNextSequenceReceiveResponse {
    const NAME: &'static str = "QueryNextSequenceReceiveResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryNextSequenceSendRequest {
    const NAME: &'static str = "QueryNextSequenceSendRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryNextSequenceSendResponse {
    const NAME: &'static str = "QueryNextSequenceSendResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketAcknowledgementRequest {
    const NAME: &'static str = "QueryPacketAcknowledgementRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketAcknowledgementResponse {
    const NAME: &'static str = "QueryPacketAcknowledgementResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketAcknowledgementsRequest {
    const NAME: &'static str = "QueryPacketAcknowledgementsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketAcknowledgementsResponse {
    const NAME: &'static str = "QueryPacketAcknowledgementsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketCommitmentRequest {
    const NAME: &'static str = "QueryPacketCommitmentRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketCommitmentResponse {
    const NAME: &'static str = "QueryPacketCommitmentResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketCommitmentsRequest {
    const NAME: &'static str = "QueryPacketCommitmentsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketCommitmentsResponse {
    const NAME: &'static str = "QueryPacketCommitmentsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketReceiptRequest {
    const NAME: &'static str = "QueryPacketReceiptRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryPacketReceiptResponse {
    const NAME: &'static str = "QueryPacketReceiptResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUnreceivedAcksRequest {
    const NAME: &'static str = "QueryUnreceivedAcksRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUnreceivedAcksResponse {
    const NAME: &'static str = "QueryUnreceivedAcksResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUnreceivedPacketsRequest {
    const NAME: &'static str = "QueryUnreceivedPacketsRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUnreceivedPacketsResponse {
    const NAME: &'static str = "QueryUnreceivedPacketsResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUpgradeErrorRequest {
    const NAME: &'static str = "QueryUpgradeErrorRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUpgradeErrorResponse {
    const NAME: &'static str = "QueryUpgradeErrorResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUpgradeRequest {
    const NAME: &'static str = "QueryUpgradeRequest";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryUpgradeResponse {
    const NAME: &'static str = "QueryUpgradeResponse";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Timeout {
    const NAME: &'static str = "Timeout";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Upgrade {
    const NAME: &'static str = "Upgrade";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for UpgradeFields {
    const NAME: &'static str = "UpgradeFields";
    const PACKAGE: &'static str = "ibc.core.channel.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.channel.v1.{}", Self::NAME)
    }
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
            ResponseResultType::Failure => "RESPONSE_RESULT_TYPE_FAILURE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RESPONSE_RESULT_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "RESPONSE_RESULT_TYPE_NOOP" => Some(Self::Noop),
            "RESPONSE_RESULT_TYPE_SUCCESS" => Some(Self::Success),
            "RESPONSE_RESULT_TYPE_FAILURE" => Some(Self::Failure),
            _ => None,
        }
    }
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
            State::Flushing => "STATE_FLUSHING",
            State::Flushcomplete => "STATE_FLUSHCOMPLETE",
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
            "STATE_FLUSHING" => Some(Self::Flushing),
            "STATE_FLUSHCOMPLETE" => Some(Self::Flushcomplete),
            _ => None,
        }
    }
}
