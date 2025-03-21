use std::{fmt, num::NonZeroU64, str::FromStr};

use enumorph::Enumorph;
use macros::{ibc_path, model};
use serde::{Deserialize, Serialize};
use subset_of::SubsetOf;
use tracing::info;
use unionlabs::{
    ibc::core::{
        channel::{
            channel::Channel, msg_acknowledgement::MsgAcknowledgement,
            msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket, msg_timeout::MsgTimeout, order::Order,
        },
        client::{
            height::Height, msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient,
        },
        connection::{
            connection_end::ConnectionEnd, msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_init::MsgConnectionOpenInit,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    never::Never,
    primitives::{Bytes, H256},
    ErrorReporter,
};
use voyager_core::{ClientType, IbcSpec, IbcSpecId, IbcStorePathKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IbcClassic {}

impl IbcSpec for IbcClassic {
    const ID: IbcSpecId = IbcSpecId::new_static(IbcSpecId::CLASSIC);

    type ClientId = ClientId;

    type StorePath = StorePath;

    type Query = Never;

    type Datagram = Datagram;

    type Event = FullEvent;

    fn update_client_datagram(client_id: Self::ClientId, client_message: Bytes) -> Self::Datagram {
        MsgUpdateClient {
            client_id,
            client_message,
        }
        .into()
    }

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(client_id: Self::ClientId, height: Height) -> Self::StorePath {
        ClientConsensusStatePath { client_id, height }.into()
    }
}

#[model]
#[derive(Hash, enumorph::Enumorph)]
pub enum StorePath {
    ClientState(ClientStatePath),
    ClientConsensusState(ClientConsensusStatePath),
    Connection(ConnectionPath),
    ChannelEnd(ChannelEndPath),
    Commitment(CommitmentPath),
    Acknowledgement(AcknowledgementPath),
    Receipt(ReceiptPath),
    NextSequenceSend(NextSequenceSendPath),
    NextSequenceRecv(NextSequenceRecvPath),
    NextSequenceAck(NextSequenceAckPath),
    NextConnectionSequence(NextConnectionSequencePath),
    NextClientSequence(NextClientSequencePath),
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientState(path) => write!(f, "{path}"),
            Self::ClientConsensusState(path) => write!(f, "{path}"),
            Self::Connection(path) => write!(f, "{path}"),
            Self::ChannelEnd(path) => write!(f, "{path}"),
            Self::Commitment(path) => write!(f, "{path}"),
            Self::Acknowledgement(path) => write!(f, "{path}"),
            Self::Receipt(path) => write!(f, "{path}"),
            Self::NextSequenceSend(path) => write!(f, "{path}"),
            Self::NextSequenceRecv(path) => write!(f, "{path}"),
            Self::NextSequenceAck(path) => write!(f, "{path}"),
            Self::NextConnectionSequence(path) => write!(f, "{path}"),
            Self::NextClientSequence(path) => write!(f, "{path}"),
        }
    }
}

impl FromStr for StorePath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self::ClientState)
            .or_else(|_| s.parse().map(Self::ClientConsensusState))
            .or_else(|_| s.parse().map(Self::Connection))
            .or_else(|_| s.parse().map(Self::ChannelEnd))
            .or_else(|_| s.parse().map(Self::Commitment))
            .or_else(|_| s.parse().map(Self::Acknowledgement))
            .or_else(|_| s.parse().map(Self::Receipt))
            .or_else(|_| s.parse().map(Self::NextSequenceSend))
            .or_else(|_| s.parse().map(Self::NextSequenceRecv))
            .or_else(|_| s.parse().map(Self::NextSequenceAck))
            .or_else(|_| s.parse().map(Self::NextConnectionSequence))
    }
}

/// The raw client state bytes as encoded by the light client.
#[ibc_path("clients/{client_id}/clientState", Bytes)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

/// The raw consensus state bytes as encoded by the light client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct ClientConsensusStatePath {
    pub client_id: ClientId,
    pub height: Height,
}

impl fmt::Display for ClientConsensusStatePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "clients/{}/consensusStates/{}-{}",
            self.client_id,
            self.height.revision(),
            self.height.height()
        )
    }
}

impl FromStr for ClientConsensusStatePath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);

        let mut it = s.split('/');

        match it.next() {
            Some(s) => {
                if s != "clients" {
                    return Err(PathParseError::InvalidStaticSegment {
                        expected: "clients",
                        found: s.to_owned(),
                    });
                }
            }
            None => return Err(PathParseError::MissingStaticSegment("clients")),
        }

        let client_id = it
            .next()
            .ok_or(PathParseError::MissingSegment)?
            .parse()
            .map_err(|e| PathParseError::Parse(ErrorReporter(e).to_string()))?;

        match it.next() {
            Some(s) => {
                if s != "consensusStates" {
                    return Err(PathParseError::InvalidStaticSegment {
                        expected: "consensusStates",
                        found: s.to_owned(),
                    });
                }
            }
            None => return Err(PathParseError::MissingStaticSegment("consensusStates")),
        }

        let height =
            Height::from_str_allow_zero_revision(it.next().ok_or(PathParseError::MissingSegment)?)
                .map_err(|e| PathParseError::Parse(ErrorReporter(e).to_string()))?;

        if it.next().is_some() {
            return Err(PathParseError::TooManySegments);
        }

        Ok(Self { client_id, height })
    }
}

impl IbcStorePathKey for ClientConsensusStatePath {
    type Spec = IbcClassic;
    type Value = Bytes;
}

#[ibc_path("connections/{connection_id:#}", ConnectionEnd)]
pub struct ConnectionPath {
    #[ibc_path(ConnectionId::from_str_prefixed)]
    pub connection_id: ConnectionId,
}

#[ibc_path("channelEnds/ports/{port_id}/channels/{channel_id:#}", Channel)]
pub struct ChannelEndPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path(
    "commitments/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}",
    H256
)]
pub struct CommitmentPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// SHA-256 of the packet acknowledgement.
///
/// If the packet has not yet been acknowledged (either because the packet does not exist or the packet has not been acknowledged yet), then the acknowledgement commitment is unset.
#[ibc_path(
    "acks/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}",
    H256
)]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

/// This defaults to `false` for packets which have not yet been received.
#[ibc_path(
    "receipts/ports/{port_id}/channels/{channel_id:#}/sequences/{sequence}",
    bool
)]
pub struct ReceiptPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
}

#[ibc_path("nextSequenceSend/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceSendPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceRecv/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceRecvPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextSequenceAck/ports/{port_id}/channels/{channel_id:#}", u64)]
pub struct NextSequenceAckPath {
    pub port_id: PortId,
    #[ibc_path(ChannelId::from_str_prefixed)]
    pub channel_id: ChannelId,
}

#[ibc_path("nextConnectionSequence", u64)]
pub struct NextConnectionSequencePath {}

#[ibc_path("nextClientSequence", u64)]
pub struct NextClientSequencePath {}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PathParseError {
    #[error("invalid static segment, expected `{expected}` but found `{found}`")]
    InvalidStaticSegment {
        expected: &'static str,
        found: String,
    },
    #[error("missing static segment `{0}`")]
    MissingStaticSegment(&'static str),
    // TODO: Figure out a way to provide more context here?
    #[error("missing segment")]
    MissingSegment,
    #[error("too many segments")]
    TooManySegments,
    // contains the stringified parse error
    #[error("error parsing segment: {0}")]
    Parse(String),
}

#[model]
#[derive(Enumorph)]
pub enum Datagram {
    CreateClient(MsgCreateClientData),
    UpdateClient(MsgUpdateClient),

    ConnectionOpenInit(MsgConnectionOpenInit),
    ConnectionOpenTry(MsgConnectionOpenTry),
    ConnectionOpenAck(MsgConnectionOpenAck),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm),

    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),

    RecvPacket(MsgRecvPacket),
    AcknowledgePacket(MsgAcknowledgement),
    TimeoutPacket(MsgTimeout),
}

impl Datagram {
    /// Returns the proof height of the IBC message, if it contains one.
    /// (ConnectionOpenInit does not contain a proof, for example)
    pub fn proof_height(&self) -> Option<Height> {
        match self {
            Datagram::CreateClient(_) => None,
            Datagram::UpdateClient(_) => None,
            Datagram::ConnectionOpenInit(_) => None,
            Datagram::ConnectionOpenTry(msg) => Some(msg.proof_height),
            Datagram::ConnectionOpenAck(msg) => Some(msg.proof_height),
            Datagram::ConnectionOpenConfirm(msg) => Some(msg.proof_height),
            Datagram::ChannelOpenInit(_) => None,
            Datagram::ChannelOpenTry(msg) => Some(msg.proof_height),
            Datagram::ChannelOpenAck(msg) => Some(msg.proof_height),
            Datagram::ChannelOpenConfirm(msg) => Some(msg.proof_height),
            Datagram::RecvPacket(msg) => Some(msg.proof_height),
            Datagram::AcknowledgePacket(msg) => Some(msg.proof_height),
            Datagram::TimeoutPacket(msg) => Some(msg.proof_height),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Datagram::CreateClient(_) => "create_client",
            Datagram::UpdateClient(_) => "update_client",
            Datagram::ConnectionOpenInit(_) => "connection_open_init",
            Datagram::ConnectionOpenTry(_) => "connection_open_try",
            Datagram::ConnectionOpenAck(_) => "connection_open_ack",
            Datagram::ConnectionOpenConfirm(_) => "connection_open_confirm",
            Datagram::ChannelOpenInit(_) => "channel_open_init",
            Datagram::ChannelOpenTry(_) => "channel_open_try",
            Datagram::ChannelOpenAck(_) => "channel_open_ack",
            Datagram::ChannelOpenConfirm(_) => "channel_open_confirm",
            Datagram::RecvPacket(_) => "recv_packet",
            Datagram::AcknowledgePacket(_) => "acknowledgement",
            Datagram::TimeoutPacket(_) => "timeout",
        }
    }
}

#[model]
pub struct CreateClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
}

#[model]
pub struct UpdateClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_heights: Vec<Height>,
}

#[model]
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[model]
pub struct ConnectionOpenTry {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[model]
pub struct ConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[model]
pub struct ConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[model]
pub struct ChannelOpenInit {
    pub port_id: PortId,
    pub channel_id: ChannelId,

    pub counterparty_port_id: PortId,

    pub connection: ConnectionEnd,

    pub version: String,
}

#[model]
pub struct ChannelOpenTry {
    pub port_id: PortId,
    pub channel_id: ChannelId,

    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,

    pub connection: ConnectionEnd,

    pub version: String,
}

#[model]
pub struct ChannelOpenAck {
    pub port_id: PortId,
    pub channel_id: ChannelId,

    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,

    pub connection: ConnectionEnd,

    pub version: String,
}

#[model]
pub struct ChannelOpenConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,

    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,

    pub connection: ConnectionEnd,

    pub version: String,
}

#[model]
pub struct WriteAcknowledgement {
    pub packet_data: Bytes,
    pub packet_ack: Bytes,
    pub packet: PacketMetadata,
}

#[model]
pub struct RecvPacket {
    pub packet_data: Bytes,
    pub packet: PacketMetadata,
}

#[model]
pub struct SendPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

#[model]
pub struct AcknowledgePacket {
    pub packet: PacketMetadata,
}

#[model]
pub struct TimeoutPacket {
    pub packet: PacketMetadata,
}

#[model]
pub struct PacketMetadata {
    pub sequence: NonZeroU64,

    pub source_channel: ChannelMetadata,
    pub destination_channel: ChannelMetadata,

    pub channel_ordering: Order,

    pub timeout_height: Height,
    pub timeout_timestamp: u64,
}

#[model]
pub struct ChannelMetadata {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    // REVIEW: Can this be different on either end of a channel?
    pub version: String,
    pub connection: ConnectionMetadata,
}

#[model]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    // this is really `Either<ConnectionId, EmptyString>`
    // REVIEW: Is it?
    pub connection_id: ConnectionId,
}

/// Similar to `IbcEvent`, but contains more information (counterparty
/// clients, channel version, etc)
#[model]
#[derive(Enumorph, SubsetOf)]
pub enum FullEvent {
    CreateClient(CreateClient),
    UpdateClient(UpdateClient),

    ConnectionOpenInit(ConnectionOpenInit),
    ConnectionOpenTry(ConnectionOpenTry),
    ConnectionOpenAck(ConnectionOpenAck),
    ConnectionOpenConfirm(ConnectionOpenConfirm),

    ChannelOpenInit(ChannelOpenInit),
    ChannelOpenTry(ChannelOpenTry),
    ChannelOpenAck(ChannelOpenAck),
    ChannelOpenConfirm(ChannelOpenConfirm),

    SendPacket(SendPacket),
    RecvPacket(RecvPacket),
    WriteAcknowledgement(WriteAcknowledgement),
    AcknowledgePacket(AcknowledgePacket),
    TimeoutPacket(TimeoutPacket),
}

impl FullEvent {
    pub fn client_id(&self) -> &ClientId {
        match self {
            Self::CreateClient(ref event) => &event.client_id,
            Self::UpdateClient(ref event) => &event.client_id,
            Self::ConnectionOpenInit(ref event) => &event.client_id,
            Self::ConnectionOpenTry(ref event) => &event.client_id,
            Self::ConnectionOpenAck(ref event) => &event.client_id,
            Self::ConnectionOpenConfirm(ref event) => &event.client_id,
            Self::ChannelOpenInit(ref event) => &event.connection.client_id,
            Self::ChannelOpenTry(ref event) => &event.connection.client_id,
            Self::ChannelOpenAck(ref event) => &event.connection.client_id,
            Self::ChannelOpenConfirm(ref event) => &event.connection.client_id,
            Self::SendPacket(ref event) => &event.packet.source_channel.connection.client_id,
            Self::RecvPacket(ref event) => &event.packet.source_channel.connection.client_id,
            Self::WriteAcknowledgement(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
            Self::AcknowledgePacket(ref event) => &event.packet.source_channel.connection.client_id,
            Self::TimeoutPacket(ref event) => &event.packet.source_channel.connection.client_id,
        }
    }

    /// Returns the counterparty client id of this ibc event, if there is a
    /// counterparty. This will return `None` for `UpdateClient` and
    /// `CreateClient`.
    pub fn counterparty_client_id(&self) -> Option<&ClientId> {
        match self {
            Self::ConnectionOpenInit(ref event) => Some(&event.counterparty_client_id),
            Self::ConnectionOpenTry(ref event) => Some(&event.counterparty_client_id),
            Self::ConnectionOpenAck(ref event) => Some(&event.counterparty_client_id),
            Self::ConnectionOpenConfirm(ref event) => Some(&event.counterparty_client_id),
            Self::ChannelOpenInit(ref event) => Some(&event.connection.counterparty.client_id),
            Self::ChannelOpenTry(ref event) => Some(&event.connection.counterparty.client_id),
            Self::ChannelOpenAck(ref event) => Some(&event.connection.counterparty.client_id),
            Self::ChannelOpenConfirm(ref event) => Some(&event.connection.counterparty.client_id),
            Self::SendPacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            Self::RecvPacket(ref event) => Some(&event.packet.source_channel.connection.client_id),
            Self::WriteAcknowledgement(ref event) => {
                Some(&event.packet.source_channel.connection.client_id)
            }
            Self::AcknowledgePacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            Self::TimeoutPacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            _ => None,
        }
    }
}

#[model]
pub struct MsgCreateClientData {
    pub msg: MsgCreateClient,
    pub client_type: ClientType,
}

pub fn log_msg(chain_id: &str, effect: &Datagram) {
    match effect.clone() {
        Datagram::ConnectionOpenInit(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                // TODO: Use Valuable here
                ?message.counterparty.connection_id,
                %message.counterparty.prefix.key_prefix,
                %message.version.identifier,
                message.version.features = %message
                    .version
                    .features
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.delay_period,
            )
        }
        Datagram::ConnectionOpenTry(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                // TODO: Use Valuable here
                ?message.counterparty.connection_id,
                %message.counterparty.prefix.key_prefix,
                %message.delay_period,
                %message.proof_height,
            )
        }
        Datagram::ConnectionOpenAck(message) => {
            info!(
                %chain_id,
                // client_state.height = message.%data.message.client_state.height(),
                %message.proof_height,
                %message.consensus_height,
                %message.connection_id,
                %message.counterparty_connection_id,
                %message.version.identifier,
                message.version.features = %message
                    .version
                    .features
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        }
        Datagram::ConnectionOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.connection_id,
                %message.proof_height,
            )
        }
        Datagram::ChannelOpenInit(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                // TODO: Use Valuable here
                ?message.channel.counterparty.channel_id,
                message.channel.connection_hops = %message
                    .channel
                    .connection_hops
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.channel.version,
            )
        }
        Datagram::ChannelOpenTry(message) => {
            info!(
                %chain_id,

                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                // TODO: Use Valuable here
                ?message.channel.counterparty.channel_id,
                message.channel.connection_hops = %message
                    .channel
                    .connection_hops
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                %message.channel.version,
                %message.counterparty_version,
                %message.proof_height,
            )
        }
        Datagram::ChannelOpenAck(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.counterparty_version,
                %message.counterparty_channel_id,
                %message.proof_height,
            )
        }
        Datagram::ChannelOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.proof_height,
            )
        }
        Datagram::RecvPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                %message.packet.data,
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
            )
        }
        Datagram::AcknowledgePacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                %message.packet.data,
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.acknowledgement,
                %message.proof_height,
            )
        }
        Datagram::TimeoutPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                %message.packet.data,
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
                %message.next_sequence_recv,
            )
        }
        Datagram::CreateClient(message) => {
            info!(
                %chain_id,
                %message.client_type,
            )
        }
        Datagram::UpdateClient(message) => {
            info!(
                %chain_id,
                %message.client_id,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::ibc::core::client::height::Height;

    use super::*;

    #[test]
    fn parse_ibc_paths_from_str() {
        assert_eq!(
            "clients/08-wasm-0/clientState"
                .parse::<StorePath>()
                .unwrap(),
            StorePath::ClientState(ClientStatePath {
                client_id: ClientId::new("08-wasm", 0)
            })
        );
        assert_eq!(
            "clients/08-wasm-0/consensusStates/0-1"
                .parse::<StorePath>()
                .unwrap(),
            StorePath::ClientConsensusState(ClientConsensusStatePath {
                client_id: ClientId::new("08-wasm", 0),
                height: Height::new(1)
            })
        );
        assert_eq!(
            "connections/connection-0".parse::<StorePath>().unwrap(),
            StorePath::Connection(ConnectionPath {
                connection_id: ConnectionId::new(0)
            })
        );
        assert_eq!(
            "channelEnds/ports/port/channels/channel-0"
                .parse::<StorePath>()
                .unwrap(),
            StorePath::ChannelEnd(ChannelEndPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0)
            })
        );
        assert_eq!(
            "commitments/ports/port/channels/channel-0/sequences/1"
                .parse::<StorePath>()
                .unwrap(),
            StorePath::Commitment(CommitmentPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<StorePath>()
                .unwrap(),
            StorePath::Acknowledgement(AcknowledgementPath {
                port_id: PortId::new("port").unwrap(),
                channel_id: ChannelId::new(0),
                sequence: 1.try_into().unwrap()
            })
        );
    }
}
