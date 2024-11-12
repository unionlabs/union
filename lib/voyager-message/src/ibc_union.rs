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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum IbcMsg {
    CreateClient(ibc_solidity::ibc::MsgCreateClient),
    UpdateClient(ibc_solidity::ibc::MsgUpdateClient),
    ConnectionOpenInit(ibc_solidity::ibc::MsgConnectionOpenInit),
    ConnectionOpenTry(ibc_solidity::ibc::MsgConnectionOpenTry),
    ConnectionOpenAck(ibc_solidity::ibc::MsgConnectionOpenAck),
    ConnectionOpenConfirm(ibc_solidity::ibc::MsgConnectionOpenConfirm),
    ChannelOpenInit(ibc_solidity::ibc::MsgChannelOpenInit),
    ChannelOpenTry(ibc_solidity::ibc::MsgChannelOpenTry),
    ChannelOpenAck(ibc_solidity::ibc::MsgChannelOpenAck),
    ChannelOpenConfirm(ibc_solidity::ibc::MsgChannelOpenConfirm),
    ChannelCloseInit(ibc_solidity::ibc::MsgChannelCloseInit),
    ChannelCloseConfirm(ibc_solidity::ibc::MsgChannelCloseConfirm),
    PacketRecv(ibc_solidity::ibc::MsgPacketRecv),
    PacketAcknowledgement(ibc_solidity::ibc::MsgPacketAcknowledgement),
    PacketTimeout(ibc_solidity::ibc::MsgPacketTimeout),
    IntentPacketRecv(ibc_solidity::ibc::MsgIntentPacketRecv),
    BatchSend(ibc_solidity::ibc::MsgBatchSend),
    BatchAcks(ibc_solidity::ibc::MsgBatchAcks),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Enumorph)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientRegistered {
    pub client_type: ClientType,
    pub client_address: H160,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientCreated {
    pub client_type: ClientType,
    pub client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientUpdated {
    pub client_type: ClientType,
    pub client_id: ClientId,
    pub height: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionOpenTry {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelOpenInit {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelOpenTry {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelOpenAck {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelOpenConfirm {
    pub port_id: H160,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecvPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
    pub relayer_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecvIntentPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub marker_maker: H160,
    pub marker_maker_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteAcknowledgement {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcknowledgePacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeoutPacket {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer: H160,
}

// metadata

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PacketMetadata {
    pub source_channel: ChannelMetadata,
    pub destination_channel: ChannelMetadata,

    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub channel_id: ChannelId,
    // REVIEW: Can this be different on either end of a channel?
    pub version: String,
    pub connection: ConnectionMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
}
