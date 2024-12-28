use enumorph::Enumorph;
use ibc_solidity::{Channel, Connection, Packet};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use unionlabs::{bytes::Bytes, hash::H256, ibc::core::client::height::Height, uint::U256};
use voyager_core::{ClientType, IbcSpec, IbcSpecId, IbcStorePathKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IbcUnion {}

impl IbcSpec for IbcUnion {
    const ID: IbcSpecId = IbcSpecId::new_static(IbcSpecId::UNION);

    type ClientId = u32;

    type StorePath = StorePath;

    type Datagram = Datagram;

    type Event = FullEvent;

    fn update_client_datagram(client_id: Self::ClientId, client_message: Bytes) -> Self::Datagram {
        Datagram::UpdateClient(MsgUpdateClient {
            client_id,
            client_message,
        })
    }

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(
        client_id: Self::ClientId,
        height: unionlabs::ibc::core::client::height::Height,
    ) -> Self::StorePath {
        ConsensusStatePath {
            client_id,
            height: height.height(),
        }
        .into()
    }
}

/// 0x0100000000000000000000000000000000000000000000000000000000000000
pub const COMMITMENT_MAGIC: H256 = {
    let mut bz = [0; 32];
    bz[0] = 1;
    H256::new(bz)
};
pub const COMMITMENT_NULL: H256 = H256::new([0; 32]);

const CLIENT_STATE: U256 = U256::from_limbs([0, 0, 0, 0]);
const CONSENSUS_STATE: U256 = U256::from_limbs([1, 0, 0, 0]);
const CONNECTIONS: U256 = U256::from_limbs([2, 0, 0, 0]);
const CHANNELS: U256 = U256::from_limbs([3, 0, 0, 0]);
const PACKETS: U256 = U256::from_limbs([4, 0, 0, 0]);
const PACKET_ACKS: U256 = U256::from_limbs([5, 0, 0, 0]);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Enumorph)]
pub enum StorePath {
    ClientState(ClientStatePath),
    ConsensusState(ConsensusStatePath),
    Connection(ConnectionPath),
    Channel(ChannelPath),
    BatchReceipts(BatchReceiptsPath),
    BatchPackets(BatchPacketsPath),
}

impl StorePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        match self {
            StorePath::ClientState(path) => path.key(),
            StorePath::ConsensusState(path) => path.key(),
            StorePath::Connection(path) => path.key(),
            StorePath::Channel(path) => path.key(),
            StorePath::BatchReceipts(path) => path.key(),
            StorePath::BatchPackets(path) => path.key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClientStatePath {
    pub client_id: u32,
}

impl ClientStatePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CLIENT_STATE.to_be_bytes())
            .chain_update(U256::from(self.client_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ClientStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConsensusStatePath {
    pub client_id: u32,
    pub height: u64,
}

impl ConsensusStatePath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CONSENSUS_STATE.to_be_bytes())
            .chain_update(U256::from(self.client_id).to_be_bytes())
            .chain_update(U256::from(self.height).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConsensusStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConnectionPath {
    pub connection_id: u32,
}

impl ConnectionPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CONNECTIONS.to_be_bytes())
            .chain_update(U256::from(self.connection_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ConnectionPath {
    type Spec = IbcUnion;

    type Value = Option<Connection>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChannelPath {
    pub channel_id: u32,
}

impl ChannelPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(CHANNELS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for ChannelPath {
    type Spec = IbcUnion;

    type Value = Option<ibc_solidity::Channel>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BatchReceiptsPath {
    pub channel_id: u32,
    pub batch_hash: H256,
}

impl BatchReceiptsPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKET_ACKS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .chain_update(self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchReceiptsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BatchPacketsPath {
    pub channel_id: u32,
    pub batch_hash: H256,
}

impl BatchPacketsPath {
    #[must_use]
    pub fn key(&self) -> H256 {
        Keccak256::new()
            .chain_update(PACKETS.to_be_bytes())
            .chain_update(U256::from(self.channel_id).to_be_bytes())
            .chain_update(self.batch_hash)
            .finalize()
            .into()
    }
}

impl IbcStorePathKey for BatchPacketsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

/// All datagrams that are a part of the IBC union specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Enumorph)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
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
    /// (ConnectionOpenInit does not contain a proof, for example)
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
    pub port_id: Bytes,
    pub counterparty_port_id: Bytes,
    pub connection_id: u32,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenTry {
    pub port_id: Bytes,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenAck {
    pub channel_id: u32,
    pub counterparty_version: String,
    pub counterparty_channel_id: u32,
    pub proof_try: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelOpenConfirm {
    pub channel_id: u32,
    pub proof_ack: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelCloseInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgChannelCloseConfirm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketRecv {
    pub packets: Vec<Packet>,
    pub relayer_msgs: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketAcknowledgement {
    pub packets: Vec<Packet>,
    pub acknowledgements: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgPacketTimeout {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgIntentPacketRecv {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgBatchSend {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgBatchAcks {}

/// The fully filled out event for IBC union. This will likely not be what is exactly emitted on chain, however *enough* information should be emitted such that this structure can be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Enumorph)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
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

type ClientId = u32;
type ConnectionId = u32;
type ChannelId = u32;

// type ClientId = NonZeroU32;
// type ConnectionId = NonZeroU32;
// type ChannelId = NonZeroU32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateClient {
    pub client_type: ClientType,
    pub client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateClient {
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
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenTry {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenAck {
    pub port_id: Bytes,
    pub channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub counterparty_channel_id: ChannelId,
    pub connection: Connection,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOpenConfirm {
    pub port_id: Bytes,
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

// TODO: Inline packet_data into PacketMetadata

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketSend {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketRecv {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub relayer_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentPacketRecv {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub market_maker_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteAck {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketAck {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,

    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketTimeout {
    pub packet_data: Bytes,

    pub packet: PacketMetadata,
}

// metadata

/// All metadata associated with a packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketMetadata {
    pub source_channel: ChannelMetadata,
    pub destination_channel: ChannelMetadata,

    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

/// All metadata associated with a Channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub channel_id: ChannelId,
    // REVIEW: Can this be different on either end of a channel?
    pub version: String,
    pub connection: ConnectionMetadata,
}

/// All metadata associated with a Connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
}

#[cfg(feature = "tracing")]
pub fn log_event(e: &FullEvent, chain_id: &voyager_core::ChainId) {
    use tracing::info;

    let event = e.name();

    match e {
        FullEvent::CreateClient(e) => info!(
            event,
            %chain_id,
            data.client_id = %e.client_id,
            data.client_type = %e.client_type,
            "event"
        ),
        FullEvent::UpdateClient(e) => info!(
            event,
            %chain_id,
            data.client_id = %e.client_id,
            data.client_type = %e.client_type,
            data.height = e.height,
            "event"
        ),
        FullEvent::ConnectionOpenInit(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            "event"
        ),
        FullEvent::ConnectionOpenTry(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ConnectionOpenAck(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ConnectionOpenConfirm(e) => info!(
            event,
            %chain_id,
            data.connection_id = %e.connection_id,
            data.client_id = %e.client_id,
            data.counterparty_client_id = %e.counterparty_client_id,
            data.counterparty_connection_id = %e.counterparty_connection_id,
            "event"
        ),
        FullEvent::ChannelOpenInit(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = %e.connection.counterparty_connection_id,
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenTry(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = %e.connection.counterparty_connection_id,
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenAck(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = %e.connection.counterparty_connection_id,
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelOpenConfirm(e) => info!(
            event,
            %chain_id,
            data.port_id = %e.port_id,
            data.channel_id = %e.channel_id,
            data.counterparty_port_id = %e.counterparty_port_id,
            data.counterparty_channel_id = %e.counterparty_channel_id,
            data.connection.state = ?e.connection.state,
            data.connection.client_id = %e.connection.client_id,
            data.connection.counterparty_client_id = %e.connection.counterparty_client_id,
            data.connection.counterparty_connection_id = %e.connection.counterparty_connection_id,
            data.version = %e.version,
            "event"
        ),
        FullEvent::ChannelCloseInit(_e) => info!(event, "event"),
        FullEvent::ChannelCloseConfirm(_e) => info!(event, "event"),
        FullEvent::PacketSend(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketRecv(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,
            data.relayer_msg = %e.relayer_msg,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::IntentPacketRecv(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,
            data.market_maker_msg = %e.market_maker_msg,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::WriteAck(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,
            data.acknowledgement = %e.acknowledgement,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketAck(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,
            data.acknowledgement = %e.acknowledgement,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
        FullEvent::PacketTimeout(e) => info!(
            event,
            %chain_id,
            data.packet_data = %e.packet_data,

            data.packet.source_channel.channel_id = %e.packet.source_channel.channel_id,
            data.packet.source_channel.version = %e.packet.source_channel.version,
            data.packet.source_channel = %e.packet.source_channel.connection.client_id,
            data.packet.source_channel = %e.packet.source_channel.connection.connection_id,

            data.packet.destination_channel.channel_id = %e.packet.destination_channel.channel_id,
            data.packet.destination_channel.version = %e.packet.destination_channel.version,
            data.packet.destination_channel = %e.packet.destination_channel.connection.client_id,
            data.packet.destination_channel = %e.packet.destination_channel.connection.connection_id,

            data.packet.timeout_height = %e.packet.timeout_height,
            data.packet.timeout_timestamp = %e.packet.timeout_timestamp,
            "event"
        ),
    }
}
