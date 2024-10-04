use std::num::NonZeroU64;

use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use subset_of::SubsetOf;
use tracing::info;
#[cfg(doc)]
use unionlabs::events::IbcEvent;
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::{
            msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
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
    ics24::{ClientConsensusStatePath, ClientStatePath, IbcPath, Path},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    traits::Member,
};

use crate::{
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta},
    PluginMessage,
};

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
    IbcEvent(ChainEvent),
    IbcMessage(IbcMessage),

    OrderedHeaders(OrderedHeaders),
    OrderedMsgUpdateClients(OrderedMsgUpdateClients),

    IdentifiedIbcMessage(WithChainId<IbcMessage>),
    IdentifiedIbcMessageBatch(WithChainId<Vec<IbcMessage>>),

    Plugin(PluginMessage),
}

impl Data {
    #[allow(clippy::result_large_err)]
    pub fn as_plugin<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        match self {
            Self::Plugin(plugin_message) => {
                plugin_message.downcast(plugin_name).map_err(Self::Plugin)
            }
            this => Err(this),
        }
    }
}

#[model]
pub struct ChainEvent {
    /// The chain where this event was emitted.
    pub chain_id: ChainId<'static>,
    /// The underlying client of this event, on [`Self::chain_id`].
    pub client_info: ClientInfo,
    /// The chain on the other end of this IBC event.
    pub counterparty_chain_id: ChainId<'static>,
    pub tx_hash: H256,
    /// The "provable height" of the event. This is the minimum height at which
    /// the effect of the IBC action that caused this event is provable in
    /// the state root of the chain identified by [`Self::chain_id`].
    pub provable_height: Height,
    pub event: FullIbcEvent,
}

impl ChainEvent {
    pub fn client_id(&self) -> &ClientId {
        match self.event {
            FullIbcEvent::CreateClient(ref event) => &event.client_id,
            FullIbcEvent::UpdateClient(ref event) => &event.client_id,
            FullIbcEvent::ConnectionOpenInit(ref event) => &event.client_id,
            FullIbcEvent::ConnectionOpenTry(ref event) => &event.client_id,
            FullIbcEvent::ConnectionOpenAck(ref event) => &event.client_id,
            FullIbcEvent::ConnectionOpenConfirm(ref event) => &event.client_id,
            FullIbcEvent::ChannelOpenInit(ref event) => &event.connection.client_id,
            FullIbcEvent::ChannelOpenTry(ref event) => &event.connection.client_id,
            FullIbcEvent::ChannelOpenAck(ref event) => &event.connection.client_id,
            FullIbcEvent::ChannelOpenConfirm(ref event) => &event.connection.client_id,
            FullIbcEvent::SendPacket(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
            FullIbcEvent::RecvPacket(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
            FullIbcEvent::WriteAcknowledgement(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
            FullIbcEvent::AcknowledgePacket(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
            FullIbcEvent::TimeoutPacket(ref event) => {
                &event.packet.source_channel.connection.client_id
            }
        }
    }

    /// Returns the counterparty client id of this ibc event, if there is a
    /// counterparty. This will return `None` for `UpdateClient` and
    /// `CreateClient`.
    pub fn counterparty_client_id(&self) -> Option<&ClientId> {
        match self.event {
            FullIbcEvent::ConnectionOpenInit(ref event) => Some(&event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenTry(ref event) => Some(&event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenAck(ref event) => Some(&event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenConfirm(ref event) => Some(&event.counterparty_client_id),
            FullIbcEvent::ChannelOpenInit(ref event) => {
                Some(&event.connection.counterparty.client_id)
            }
            FullIbcEvent::ChannelOpenTry(ref event) => {
                Some(&event.connection.counterparty.client_id)
            }
            FullIbcEvent::ChannelOpenAck(ref event) => {
                Some(&event.connection.counterparty.client_id)
            }
            FullIbcEvent::ChannelOpenConfirm(ref event) => {
                Some(&event.connection.counterparty.client_id)
            }
            FullIbcEvent::SendPacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            FullIbcEvent::RecvPacket(ref event) => {
                Some(&event.packet.source_channel.connection.client_id)
            }
            FullIbcEvent::WriteAcknowledgement(ref event) => {
                Some(&event.packet.source_channel.connection.client_id)
            }
            FullIbcEvent::AcknowledgePacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            FullIbcEvent::TimeoutPacket(ref event) => {
                Some(&event.packet.destination_channel.connection.client_id)
            }
            _ => None,
        }
    }
}

#[model]
#[derive(Enumorph)]
pub enum IbcMessage {
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

impl IbcMessage {
    /// Returns the proof height of the IBC message, if it has one.
    /// (ConnectionOpenInit does not contain a proof, for example)
    pub fn proof_height(&self) -> Option<Height> {
        match self {
            IbcMessage::CreateClient(_) => None,
            IbcMessage::UpdateClient(_) => None,
            IbcMessage::ConnectionOpenInit(_) => None,
            IbcMessage::ConnectionOpenTry(msg) => Some(msg.proof_height),
            IbcMessage::ConnectionOpenAck(msg) => Some(msg.proof_height),
            IbcMessage::ConnectionOpenConfirm(msg) => Some(msg.proof_height),
            IbcMessage::ChannelOpenInit(_) => None,
            IbcMessage::ChannelOpenTry(msg) => Some(msg.proof_height),
            IbcMessage::ChannelOpenAck(msg) => Some(msg.proof_height),
            IbcMessage::ChannelOpenConfirm(msg) => Some(msg.proof_height),
            IbcMessage::RecvPacket(msg) => Some(msg.proof_height),
            IbcMessage::AcknowledgePacket(msg) => Some(msg.proof_height),
            IbcMessage::TimeoutPacket(msg) => Some(msg.proof_height),
        }
    }
}

#[model]
pub struct CreateClient {
    pub client_id: ClientId,
    pub client_type: ClientType<'static>,
    pub consensus_height: Height,
}

#[model]
pub struct UpdateClient {
    pub client_id: ClientId,
    pub client_type: ClientType<'static>,
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
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub packet_data: Vec<u8>,

    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub packet_ack: Vec<u8>,

    pub packet: PacketMetadata,
}

#[model]
pub struct RecvPacket {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub packet_data: Vec<u8>,

    pub packet: PacketMetadata,
}

#[model]
pub struct SendPacket {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub packet_data: Vec<u8>,

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

/// Similar to [`IbcEvent`], but contains more information (counterparty
/// clients, channel version, etc)
#[model]
#[derive(Enumorph, SubsetOf)]
pub enum FullIbcEvent {
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

#[model]
pub struct UnfinalizedTrustedClientState {
    pub height: Height,
    pub client_state: ClientStateMeta,
}

#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcState<P: IbcPath> {
    pub chain_id: ChainId<'static>,
    pub path: P,
    /// The height that the state was read at.
    pub height: Height,
    pub state: P::Value,
}

#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcProof<P: IbcPath> {
    pub path: P,
    pub height: Height,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof: Vec<u8>,
}

#[model]
pub struct RawIbcProof {
    pub path: Path,
    pub height: Height,
    /// The raw proof, encoded as JSON, which will be encoded by the relevant
    /// client module.
    pub proof: Value,
}

#[model]
pub struct DecodedClientStateMeta {
    pub path: ClientStatePath,
    /// The height that the state was read at. Same as [`IbcState::height`].
    pub height: Height,
    pub state: ClientStateMeta,
}

#[model]
pub struct DecodedConsensusStateMeta {
    pub path: ClientConsensusStatePath,
    pub height: Height,
    pub state: ConsensusStateMeta,
}

#[model]
pub struct DecodedHeaderMeta {
    /// The new trusted height that the header provides a consensus update to.
    pub height: Height,
}

#[model]
pub struct OrderedHeaders {
    pub headers: Vec<(DecodedHeaderMeta, Value)>,
}

#[model]
pub struct OrderedMsgUpdateClients {
    pub updates: Vec<(DecodedHeaderMeta, MsgUpdateClient)>,
}

#[model]
pub struct EncodedClientState {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub encoded_client_state: Vec<u8>,
}

#[model]
pub struct EncodedConsensusState {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub encoded_consensus_state: Vec<u8>,
}

#[model]
pub struct EncodedHeader {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub encoded_header: Vec<u8>,
}

#[model]
pub struct WithChainId<T> {
    pub chain_id: ChainId<'static>,
    pub message: T,
}

#[model]
pub struct MsgCreateClientData {
    pub msg: MsgCreateClient,
    pub client_type: ClientType<'static>,
}

pub fn log_msg(chain_id: &str, effect: &IbcMessage) {
    match effect.clone() {
        IbcMessage::ConnectionOpenInit(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                message.counterparty.connection_id = %message.counterparty.connection_id.as_deref().unwrap_or_default(),
                message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
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
        IbcMessage::ConnectionOpenTry(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                message.counterparty.connection_id = %message.counterparty.connection_id.as_deref().unwrap_or_default(),
                message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                %message.delay_period,
                // TODO: This needs `valuable`
                // message.counterparty_versions = %message
                //     .counterparty_versions
                //     .into_iter()
                //     .map(Into::into)
                //     .collect(),
                %message.proof_height,
                %message.consensus_height,
            )
        }
        IbcMessage::ConnectionOpenAck(message) => {
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
        IbcMessage::ConnectionOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.connection_id,
                %message.proof_height,
            )
        }
        IbcMessage::ChannelOpenInit(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                %message.channel.counterparty.channel_id,
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
        IbcMessage::ChannelOpenTry(message) => {
            info!(
                %chain_id,

                %message.port_id,
                %message.channel.state,
                %message.channel.ordering,
                %message.channel.counterparty.port_id,
                %message.channel.counterparty.channel_id,
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
        IbcMessage::ChannelOpenAck(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.counterparty_version,
                %message.counterparty_channel_id,
                %message.proof_height,
            )
        }
        IbcMessage::ChannelOpenConfirm(message) => {
            info!(
                %chain_id,
                %message.port_id,
                %message.channel_id,
                %message.proof_height,
            )
        }
        IbcMessage::RecvPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
            )
        }
        IbcMessage::AcknowledgePacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                message.data = %::serde_utils::to_hex(message.acknowledgement),
                %message.proof_height,
            )
        }
        IbcMessage::TimeoutPacket(message) => {
            info!(
                %chain_id,
                %message.packet.sequence,
                %message.packet.source_port,
                %message.packet.source_channel,
                %message.packet.destination_port,
                %message.packet.destination_channel,
                message.data = %::serde_utils::to_hex(message.packet.data),
                %message.packet.timeout_height,
                %message.packet.timeout_timestamp,

                %message.proof_height,
                %message.next_sequence_recv,
            )
        }
        IbcMessage::CreateClient(message) => {
            info!(
                %chain_id,
                %message.client_type,
            )
        }
        IbcMessage::UpdateClient(message) => {
            info!(
                %chain_id,
                %message.client_id,
            )
        }
    }
}
