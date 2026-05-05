use cosmwasm_event::Event;
use cosmwasm_std::Addr;
use depolama::Bytes;
use ibc_union_spec::{ChannelId, ClientId, ConnectionId, Timestamp};
use unionlabs::primitives::{H256, encoding::HexUnprefixed};

#[derive(Event)]
#[event("register_client")]
pub struct RegisterClient {
    pub client_type: String,
    pub client_address: Addr,
}

#[derive(Event)]
#[event("create_client")]
pub struct CreateClient {
    pub client_id: ClientId,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

#[derive(Event)]
#[event("update_client")]
pub struct UpdateClient {
    pub client_id: ClientId,
    pub counterparty_height: u64,
}

#[derive(Event)]
#[event("force_update_client")]
pub struct ForceUpdateClient {
    pub client_id: ClientId,
    pub counterparty_height: u64,
}

#[derive(Event)]
#[event("misbehaviour")]
pub struct Misbehaviour {
    pub client_id: ClientId,
}

#[derive(Event)]
#[event("connection_open_init")]
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Event)]
#[event("connection_open_try")]
pub struct ConnectionOpenTry {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Event)]
#[event("connection_open_ack")]
pub struct ConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Event)]
#[event("connection_open_confirm")]
pub struct ConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Event)]
#[event("channel_open_init")]
pub struct ChannelOpenInit<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub connection_id: ConnectionId,
    pub version: &'a str,
}

#[derive(Event)]
#[event("channel_open_try")]
pub struct ChannelOpenTry<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
    pub counterparty_version: &'a str,
}

#[derive(Event)]
#[event("channel_open_ack")]
pub struct ChannelOpenAck<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

#[derive(Event)]
#[event("channel_open_confirm")]
pub struct ChannelOpenConfirm<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

#[derive(Event)]
#[event("channel_close_init")]
pub struct ChannelCloseInit<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub counterparty_channel_id: ChannelId,
}

#[derive(Event)]
#[event("channel_close_confirm")]
pub struct ChannelCloseConfirm<'a> {
    pub port_id: &'a Addr,
    pub channel_id: ChannelId,
    pub counterparty_port_id: &'a Bytes<HexUnprefixed>,
    pub counterparty_channel_id: ChannelId,
}

#[derive(Event)]
#[event("packet_send")]
pub struct PacketSend {
    pub packet_source_channel_id: ChannelId,
    pub packet_destination_channel_id: ChannelId,
    pub packet_data: Bytes,
    pub packet_timeout_height: u64,
    pub packet_timeout_timestamp: Timestamp,
    pub channel_id: ChannelId,
    pub packet_hash: H256,
}

#[derive(Event)]
#[event("batch_send")]
pub struct BatchSend {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub batch_hash: H256,
}

#[derive(Event)]
#[event("batch_acks")]
pub struct BatchAcks {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub batch_hash: H256,
}

#[derive(Event)]
#[event("packet_recv")]
pub struct PacketRecv<'a> {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub maker: &'a Addr,
    pub maker_msg: &'a Bytes<HexUnprefixed>,
}

#[derive(Event)]
#[event("intent_packet_recv")]
pub struct IntentPacketRecv<'a> {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub maker: &'a Addr,
    pub maker_msg: &'a Bytes<HexUnprefixed>,
}

#[derive(Event)]
#[event("packet_ack")]
pub struct PacketAck<'a> {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub acknowledgement: &'a Bytes<HexUnprefixed>,
    pub maker: &'a Addr,
}

#[derive(Event)]
#[event("write_ack")]
pub struct WriteAck<'a> {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub acknowledgement: &'a Bytes<HexUnprefixed>,
}

#[derive(Event)]
#[event("timeout_packet")]
pub struct TimeoutPacket<'a> {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
    pub maker: &'a Addr,
}

#[derive(Event)]
#[event("commit_membership_proof")]
pub struct CommitMembershipProof<'a> {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub path: &'a Bytes<HexUnprefixed>,
    pub value: &'a Bytes<HexUnprefixed>,
}

#[derive(Event)]
#[event("commit_non_membership_proof")]
pub struct CommitNonMembershipProof<'a> {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub path: &'a Bytes<HexUnprefixed>,
}

#[derive(Event)]
#[event("create_lens_client")]
pub struct CreateLensClient<'a> {
    pub client_id: ClientId,
    pub l1_client_id: ClientId,
    pub l2_client_id: ClientId,
    pub l2_chain_id: &'a str,
}
