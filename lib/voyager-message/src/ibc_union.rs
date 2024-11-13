use enumorph::Enumorph;
use ibc_solidity::ibc::Connection;
use serde::{Deserialize, Serialize};
use unionlabs::{
    bytes::Bytes,
    hash::{H160, H256},
    ics24::ethabi::{
        BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath, ConnectionPath,
        ConsensusStatePath, Path,
    },
};
use voyager_core::{ClientType, IbcVersionId};

use crate::{IbcSpec, IbcStorePathKey};

pub enum IbcUnion {}

impl IbcSpec for IbcUnion {
    const ID: IbcVersionId = IbcVersionId::new_static(IbcVersionId::UNION);

    type ClientId = u32;

    type StorePath = Path;

    type Datagram = IbcMsg;

    type Event = FullIbcEvent;

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        unionlabs::ics24::ethabi::ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(
        client_id: Self::ClientId,
        height: unionlabs::ibc::core::client::height::Height,
    ) -> Self::StorePath {
        unionlabs::ics24::ethabi::ConsensusStatePath {
            client_id,
            height: height.height(),
        }
        .into()
    }
}

impl IbcStorePathKey for ClientStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

impl IbcStorePathKey for ConsensusStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

impl IbcStorePathKey for ConnectionPath {
    type Spec = IbcUnion;

    type Value = Option<Connection>;
}

impl IbcStorePathKey for ChannelPath {
    type Spec = IbcUnion;

    type Value = Option<ibc_solidity::ibc::Channel>;
}

impl IbcStorePathKey for BatchReceiptsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

impl IbcStorePathKey for BatchPacketsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Enumorph)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum IbcMsg {
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

impl IbcMsg {
    pub fn name(&self) -> &'static str {
        match self {
            IbcMsg::CreateClient(_) => "create_client",
            IbcMsg::UpdateClient(_) => "update_client",
            IbcMsg::ConnectionOpenInit(_) => "connection_open_init",
            IbcMsg::ConnectionOpenTry(_) => "connection_open_try",
            IbcMsg::ConnectionOpenAck(_) => "connection_open_ack",
            IbcMsg::ConnectionOpenConfirm(_) => "connection_open_confirm",
            IbcMsg::ChannelOpenInit(_) => "channel_open_init",
            IbcMsg::ChannelOpenTry(_) => "channel_open_try",
            IbcMsg::ChannelOpenAck(_) => "channel_open_ack",
            IbcMsg::ChannelOpenConfirm(_) => "channel_open_confirm",
            IbcMsg::ChannelCloseInit(_) => "channel_close_init",
            IbcMsg::ChannelCloseConfirm(_) => "channel_close_confirm",
            IbcMsg::PacketRecv(_) => "packet_recv",
            IbcMsg::PacketAcknowledgement(_) => "packet_acknowledgement",
            IbcMsg::PacketTimeout(_) => "packet_timeout",
            IbcMsg::IntentPacketRecv(_) => "intent_packet_recv",
            IbcMsg::BatchSend(_) => "batch_send",
            IbcMsg::BatchAcks(_) => "batch_acks",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgCreateClient {
    pub client_type: ClientType,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgUpdateClient {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenTry {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenAck {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenConfirm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenTry {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenAck {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenConfirm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketRecv {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketAcknowledgement {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketTimeout {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgIntentPacketRecv {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgBatchSend {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgBatchAcks {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Enumorph)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum FullIbcEvent {
    ClientRegistered(ClientRegistered),
    ClientCreated(ClientCreated),
    ClientUpdated(ClientUpdated),
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
    SendPacket(SendPacket),
    RecvPacket(RecvPacket),
    RecvIntentPacket(RecvIntentPacket),
    WriteAcknowledgement(WriteAcknowledgement),
    AcknowledgePacket(AcknowledgePacket),
    TimeoutPacket(TimeoutPacket),
}

type ClientId = u32;
type ConnectionId = u32;
type ChannelId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientRegistered {
    pub client_type: ClientType,
    pub client_address: H160,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCreated {
    pub client_type: ClientType,
    pub client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientUpdated {
    pub client_type: ClientType,
    pub client_id: ClientId,
    pub height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionOpenTry {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenInit {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenTry {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenAck {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenConfirm {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecvPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
    pub relayer_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecvIntentPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub marker_maker: H160,
    pub marker_maker_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteAcknowledgement {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgePacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
}

// metadata

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketMetadata {
    pub source_channel: ChannelMetadata,
    pub destination_channel: ChannelMetadata,

    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub channel_id: ChannelId,
    // REVIEW: Can this be different on either end of a channel?
    pub version: String,
    pub connection: ConnectionMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
}
