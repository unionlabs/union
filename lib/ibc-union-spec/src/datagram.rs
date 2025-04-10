use enumorph::Enumorph;
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};
use voyager_primitives::ClientType;

use crate::{
    types::{ChannelId, ClientId, ConnectionId},
    Channel, Packet,
};

/// All datagrams that are a part of the IBC union specification.
#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "@type", content = "@value", rename_all = "snake_case")
)]
pub enum Datagram {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    ConnectionOpenInit(MsgConnectionOpenInit),
    ConnectionOpenTry(MsgConnectionOpenTry),
    ConnectionOpenAck(MsgConnectionOpenAck),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm),
    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),
    ChannelCloseInit(MsgChannelCloseInit),
    ChannelCloseConfirm(MsgChannelCloseConfirm),
    PacketRecv(MsgPacketRecv),
    PacketAcknowledgement(MsgPacketAcknowledgement),
    PacketTimeout(MsgPacketTimeout),
    IntentPacketRecv(MsgIntentPacketRecv),
    BatchSend(MsgBatchSend),
    BatchAcks(MsgBatchAcks),
}

impl Datagram {
    /// Returns the proof height of the IBC message, if it has one.
    /// ([`MsgConnectionOpenInit`] does not contain a proof, for example)
    pub fn proof_height(&self) -> Option<Height> {
        match self {
            Self::CreateClient(_) => None,
            Self::UpdateClient(_) => None,
            Self::ConnectionOpenInit(_) => None,
            Self::ConnectionOpenTry(msg) => Some(Height::new(msg.proof_height)),
            Self::ConnectionOpenAck(msg) => Some(Height::new(msg.proof_height)),
            Self::ConnectionOpenConfirm(msg) => Some(Height::new(msg.proof_height)),
            Self::ChannelOpenInit(_) => None,
            Self::ChannelOpenTry(msg) => Some(Height::new(msg.proof_height)),
            Self::ChannelOpenAck(msg) => Some(Height::new(msg.proof_height)),
            Self::ChannelOpenConfirm(msg) => Some(Height::new(msg.proof_height)),
            Self::ChannelCloseInit(_msg) => todo!(),
            Self::ChannelCloseConfirm(_msg) => todo!(),
            Self::PacketRecv(msg) => Some(Height::new(msg.proof_height)),
            Self::PacketAcknowledgement(msg) => Some(Height::new(msg.proof_height)),
            Self::PacketTimeout(_msg) => todo!(),
            Self::IntentPacketRecv(_msg) => todo!(),
            Self::BatchSend(_msg) => todo!(),
            Self::BatchAcks(_msg) => todo!(),
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
            Self::PacketRecv(_) => "packet_recv",
            Self::PacketAcknowledgement(_) => "packet_acknowledgement",
            Self::PacketTimeout(_) => "packet_timeout",
            Self::IntentPacketRecv(_) => "intent_packet_recv",
            Self::BatchSend(_) => "batch_send",
            Self::BatchAcks(_) => "batch_acks",
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
pub struct MsgCreateClient {
    pub client_type: ClientType,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgConnectionOpenInit {
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
pub struct MsgConnectionOpenTry {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
    pub proof_init: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub proof_try: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelOpenInit {
    pub port_id: Bytes,
    pub counterparty_port_id: Bytes,
    pub connection_id: ConnectionId,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelOpenTry {
    pub port_id: Bytes,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelOpenAck {
    pub channel_id: ChannelId,
    pub counterparty_version: String,
    pub counterparty_channel_id: ChannelId,
    pub proof_try: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelOpenConfirm {
    pub channel_id: ChannelId,
    pub proof_ack: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgPacketRecv {
    pub packets: Vec<Packet>,
    pub relayer_msgs: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgPacketAcknowledgement {
    pub packets: Vec<Packet>,
    pub acknowledgements: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgPacketTimeout {
    pub packet: Packet,
    /// TODO: Should this be proof_unreceived?
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgIntentPacketRecv {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgBatchSend {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct MsgBatchAcks {}
