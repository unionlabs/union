use enumorph::Enumorph;
use unionlabs::primitives::Bytes;
use voyager_primitives::{ClientType, Timestamp};

use crate::{
    types::{ChannelId, ClientId, ConnectionId},
    Connection, Packet,
};

/// The fully filled out event for IBC union. This will likely not be what is exactly emitted on chain, however *enough* information should be emitted such that this structure can be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "@type", content = "@value", rename_all = "snake_case")
)]
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

    ChannelCloseInit(ChannelCloseInit),
    ChannelCloseConfirm(ChannelCloseConfirm),

    PacketSend(PacketSend),
    PacketRecv(PacketRecv),
    IntentPacketRecv(IntentPacketRecv),
    WriteAck(WriteAck),
    PacketAck(PacketAck),
    PacketTimeout(PacketTimeout),
}

impl FullEvent {
    pub fn counterparty_client_id(&self) -> Option<ClientId> {
        match self {
            Self::CreateClient(_) => None,
            Self::UpdateClient(_) => None,
            Self::ConnectionOpenInit(event) => Some(event.counterparty_client_id),
            Self::ConnectionOpenTry(event) => Some(event.counterparty_client_id),
            Self::ConnectionOpenAck(event) => Some(event.counterparty_client_id),
            Self::ConnectionOpenConfirm(event) => Some(event.counterparty_client_id),
            Self::ChannelOpenInit(event) => Some(event.connection.counterparty_client_id),
            Self::ChannelOpenTry(event) => Some(event.connection.counterparty_client_id),
            Self::ChannelOpenAck(event) => Some(event.connection.counterparty_client_id),
            Self::ChannelOpenConfirm(event) => Some(event.connection.counterparty_client_id),
            Self::ChannelCloseInit(_) => todo!(),
            Self::ChannelCloseConfirm(_) => todo!(),
            Self::PacketSend(event) => Some(event.packet.destination_channel.connection.client_id),
            Self::PacketRecv(event) => Some(event.packet.source_channel.connection.client_id),
            Self::IntentPacketRecv(event) => Some(event.packet.source_channel.connection.client_id),
            Self::WriteAck(event) => Some(event.packet.source_channel.connection.client_id),
            Self::PacketAck(event) => Some(event.packet.destination_channel.connection.client_id),
            Self::PacketTimeout(event) => {
                Some(event.packet.destination_channel.connection.client_id)
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateClient(_) => "create_client",
            Self::UpdateClient(_) => "update_client",
            Self::ConnectionOpenInit(_) => "connection_open_init",
            Self::ConnectionOpenTry(_) => "connection_open_try",
            Self::ConnectionOpenAck(_) => "connection_open_ack",
            Self::ConnectionOpenConfirm(_) => "connection_open_confirm",
            Self::ChannelOpenInit(_) => "channel_open_init",
            Self::ChannelOpenTry(_) => "channel_open_try",
            Self::ChannelOpenAck(_) => "channel_open_ack",
            Self::ChannelOpenConfirm(_) => "channel_open_confirm",
            Self::ChannelCloseInit(_) => "channel_close_init",
            Self::ChannelCloseConfirm(_) => "channel_close_confirm",
            Self::PacketSend(_) => "packet_send",
            Self::PacketRecv(_) => "packet_recv",
            Self::IntentPacketRecv(_) => "intent_packet_recv",
            Self::WriteAck(_) => "write_ack",
            Self::PacketAck(_) => "packet_ack",
            Self::PacketTimeout(_) => "packet_timeout",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct CreateClient {
    pub client_type: ClientType,
    pub client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct UpdateClient {
    pub client_type: ClientType,
    pub client_id: ClientId,
    pub height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionOpenTry {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelOpenInit {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelOpenTry {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelOpenAck {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelOpenConfirm {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketSend {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

macro_rules! packet_method {
    () => {
        /// Construct the [`Packet`] for this event.
        pub fn packet(&self) -> Packet {
            Packet {
                source_channel_id: self.packet.source_channel.channel_id,
                destination_channel_id: self.packet.destination_channel.channel_id,
                data: self.packet_data.clone(),
                timeout_height: self.packet.timeout_height,
                timeout_timestamp: self.packet.timeout_timestamp,
            }
        }
    };
}

impl PacketSend {
    packet_method!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketRecv {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub maker_msg: Bytes,
}

impl PacketRecv {
    packet_method!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct IntentPacketRecv {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub market_maker_msg: Bytes,
}

impl IntentPacketRecv {
    packet_method!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct WriteAck {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

impl WriteAck {
    packet_method!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketAck {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

impl PacketAck {
    packet_method!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketTimeout {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

impl PacketTimeout {
    packet_method!();
}

/// All metadata associated with a packet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketMetadata {
    pub source_channel: ChannelMetadata,
    pub destination_channel: ChannelMetadata,

    pub timeout_height: u64,
    pub timeout_timestamp: Timestamp,
}

/// All metadata associated with a Channel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ChannelMetadata {
    pub channel_id: ChannelId,
    // REVIEW: Can this be different on either end of a channel?
    pub version: String,
    pub connection: ConnectionMetadata,
}

/// All metadata associated with a Connection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
}
