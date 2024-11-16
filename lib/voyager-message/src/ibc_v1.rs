use std::num::NonZeroU64;

use enumorph::Enumorph;
use macros::model;
use serde::{Deserialize, Serialize};
use subset_of::SubsetOf;
use tracing::info;
use unionlabs::{
    bytes::Bytes,
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
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextConnectionSequencePath, NextSequenceAckPath,
        NextSequenceRecvPath, NextSequenceSendPath, ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
};
use voyager_core::{ClientType, IbcVersionId};

use crate::{IbcSpec, IbcStorePathKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IbcV1 {}

impl IbcSpec for IbcV1 {
    const ID: IbcVersionId = IbcVersionId::new_static(IbcVersionId::V1_0_0);

    type ClientId = ClientId;
    // type Height = Height;

    type StorePath = unionlabs::ics24::Path;

    type Datagram = IbcMessage;

    type Event = FullIbcEvent;

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        unionlabs::ics24::ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(client_id: Self::ClientId, height: Height) -> Self::StorePath {
        unionlabs::ics24::ClientConsensusStatePath { client_id, height }.into()
    }
}

macro_rules! impl_ibc_store_path_key_via_ibc_path {
    ($($ty:ty,)*) => {
        $(
            impl IbcStorePathKey for $ty {
                type Spec = IbcV1;

                type Value = <Self as IbcPath>::Value;
            }
        )*
    };
}

impl_ibc_store_path_key_via_ibc_path!(
    ClientStatePath,
    ClientConsensusStatePath,
    ConnectionPath,
    ChannelEndPath,
    CommitmentPath,
    AcknowledgementPath,
    ReceiptPath,
    NextSequenceSendPath,
    NextSequenceRecvPath,
    NextSequenceAckPath,
    NextConnectionSequencePath,
);

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

    pub fn name(&self) -> &'static str {
        match self {
            IbcMessage::CreateClient(_) => "create_client",
            IbcMessage::UpdateClient(_) => "update_client",
            IbcMessage::ConnectionOpenInit(_) => "connection_open_init",
            IbcMessage::ConnectionOpenTry(_) => "connection_open_try",
            IbcMessage::ConnectionOpenAck(_) => "connection_open_ack",
            IbcMessage::ConnectionOpenConfirm(_) => "connection_open_confirm",
            IbcMessage::ChannelOpenInit(_) => "channel_open_init",
            IbcMessage::ChannelOpenTry(_) => "channel_open_try",
            IbcMessage::ChannelOpenAck(_) => "channel_open_ack",
            IbcMessage::ChannelOpenConfirm(_) => "channel_open_confirm",
            IbcMessage::RecvPacket(_) => "recv_packet",
            IbcMessage::AcknowledgePacket(_) => "acknowledgement",
            IbcMessage::TimeoutPacket(_) => "timeout",
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
pub enum FullIbcEvent {
    // TODO: Probably move create client and update client into the top level message enum
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

impl FullIbcEvent {
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

pub fn log_msg(chain_id: &str, effect: &IbcMessage) {
    match effect.clone() {
        IbcMessage::ConnectionOpenInit(message) => {
            info!(
                %chain_id,
                %message.client_id,
                %message.counterparty.client_id,
                // TODO: Use Valuable here
                ?message.counterparty.connection_id,
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
                // TODO: Use Valuable here
                ?message.counterparty.connection_id,
                message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                %message.delay_period,
                %message.proof_height,
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
        IbcMessage::ChannelOpenTry(message) => {
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
