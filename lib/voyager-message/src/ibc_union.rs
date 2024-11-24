use enumorph::Enumorph;
use ibc_solidity::ibc::{Channel, Connection};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bytes::Bytes,
    hash::{H160, H256},
    ibc::core::client::height::Height,
    ics24::ethabi::{
        BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath, ConnectionPath,
        ConsensusStatePath, Path,
    },
};
use voyager_core::{ClientType, IbcVersionId};

use crate::{IbcSpec, IbcStorePathKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IbcUnion {}

impl IbcSpec for IbcUnion {
    const ID: IbcVersionId = IbcVersionId::new_static(IbcVersionId::UNION);

    type ClientId = u32;

    type StorePath = Path;

    type Datagram = IbcMsg;

    type Event = FullIbcEvent;

    fn update_client_datagram(client_id: Self::ClientId, client_message: Bytes) -> Self::Datagram {
        IbcMsg::UpdateClient(MsgUpdateClient {
            client_id,
            client_message,
        })
    }

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
    /// Returns the proof height of the IBC message, if it has one.
    /// (ConnectionOpenInit does not contain a proof, for example)
    pub fn proof_height(&self) -> Option<Height> {
        match self {
            Self::CreateClient(_msg) => None,
            Self::UpdateClient(_msg) => None,
            Self::ConnectionOpenInit(_msg) => None,
            Self::ConnectionOpenTry(msg) => Some(Height::new(msg.proof_height)),
            Self::ConnectionOpenAck(msg) => Some(Height::new(msg.proof_height)),
            Self::ConnectionOpenConfirm(msg) => Some(Height::new(msg.proof_height)),
            Self::ChannelOpenInit(_msg) => todo!(),
            Self::ChannelOpenTry(_msg) => todo!(),
            Self::ChannelOpenAck(_msg) => todo!(),
            Self::ChannelOpenConfirm(_msg) => todo!(),
            Self::ChannelCloseInit(_msg) => todo!(),
            Self::ChannelCloseConfirm(_msg) => todo!(),
            Self::PacketRecv(_msg) => todo!(),
            Self::PacketAcknowledgement(_msg) => todo!(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgCreateClient {
    pub client_type: ClientType,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgUpdateClient {
    pub client_id: u32,
    pub client_message: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenInit {
    pub client_id: u32,
    pub counterparty_client_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenTry {
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
    pub proof_init: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenAck {
    pub connection_id: u32,
    pub counterparty_connection_id: u32,
    pub proof_try: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: u32,
    pub proof_ack: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenInit {
    port_id: String,
    counterparty_port_id: Bytes,
    connection_id: u32,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenTry {
    port_id: String,
    channel: Channel,
    counterparty_version: String,
    proof_init: Bytes,
    proof_height: u64,
}

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

impl FullIbcEvent {
    pub fn counterparty_client_id(&self) -> Option<ClientId> {
        match self {
            FullIbcEvent::ClientRegistered(_) => None,
            FullIbcEvent::ClientCreated(_) => None,
            FullIbcEvent::ClientUpdated(_) => None,
            FullIbcEvent::ConnectionOpenInit(event) => Some(event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenTry(event) => Some(event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenAck(event) => Some(event.counterparty_client_id),
            FullIbcEvent::ConnectionOpenConfirm(event) => Some(event.counterparty_client_id),
            FullIbcEvent::ChannelOpenInit(event) => Some(event.connection.counterparty_client_id),
            FullIbcEvent::ChannelOpenTry(event) => Some(event.connection.counterparty_client_id),
            FullIbcEvent::ChannelOpenAck(event) => Some(event.connection.counterparty_client_id),
            FullIbcEvent::ChannelOpenConfirm(event) => {
                Some(event.connection.counterparty_client_id)
            }
            FullIbcEvent::ChannelCloseInit(_) => todo!(),
            FullIbcEvent::ChannelCloseConfirm(_) => todo!(),
            Self::SendPacket(event) => Some(event.packet.destination_channel.connection.client_id),
            Self::RecvPacket(event) => Some(event.packet.source_channel.connection.client_id),
            Self::RecvIntentPacket(event) => Some(event.packet.source_channel.connection.client_id),
            Self::WriteAcknowledgement(event) => {
                Some(event.packet.source_channel.connection.client_id)
            }
            Self::AcknowledgePacket(event) => {
                Some(event.packet.destination_channel.connection.client_id)
            }
            Self::TimeoutPacket(event) => {
                Some(event.packet.destination_channel.connection.client_id)
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use voyager_core::ChainId;

    use super::*;
    use crate::{
        data::{IbcDatagram, WithChainId},
        into_value, VoyagerMessage,
    };

    #[test]
    fn print() {
        let op = voyager_vm::data::<VoyagerMessage>(WithChainId {
            chain_id: ChainId::new("union-devnet-1"),
            message: IbcDatagram::new::<IbcUnion>(IbcMsg::ConnectionOpenInit(
                MsgConnectionOpenInit {
                    client_id: 0,
                    counterparty_client_id: 0,
                },
            )),
        });

        println!("{}", into_value(op));
    }
}
