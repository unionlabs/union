use starknet::ContractAddress;
use crate::types::*;

#[derive(Drop, Serde)]
pub struct MsgMigrateState {
    pub client_id: ClientId,
    pub client_state: ByteArray,
    pub consensus_state: ByteArray,
    pub height: u64,
}

#[derive(Drop, Serde)]
pub struct MsgRegisterClient {
    pub client_type: ByteArray,
    pub client_address: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgCreateClient {
    pub client_type: ByteArray,
    pub client_state_bytes: ByteArray,
    pub consensus_state_bytes: ByteArray,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: ByteArray,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgForceUpdateClient {
    pub client_id: ClientId,
    pub client_state_bytes: ByteArray,
    pub consensus_state_bytes: ByteArray,
}

#[derive(Drop, Serde)]
pub struct MsgConnectionOpenInit {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Drop, Serde)]
pub struct MsgConnectionOpenTry {
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
    pub client_id: ClientId,
    pub proof_init: ByteArray,
    pub proof_height: u64,
}

#[derive(Drop, Serde)]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub proof_try: ByteArray,
    pub proof_height: u64,
}

#[derive(Drop, Serde)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: ByteArray,
    pub proof_height: u64,
}

#[derive(Drop, Serde)]
pub struct MsgChannelOpenInit {
    pub port_id: ContractAddress,
    pub counterparty_port_id: ByteArray,
    pub connection_id: ConnectionId,
    pub version: ByteArray,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgChannelOpenTry {
    pub port_id: ContractAddress,
    pub channel: Channel,
    pub counterparty_version: ByteArray,
    pub proof_init: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgChannelOpenAck {
    pub channel_id: ChannelId,
    pub counterparty_version: ByteArray,
    pub counterparty_channel_id: ChannelId,
    pub proof_try: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgChannelOpenConfirm {
    pub channel_id: ChannelId,
    pub proof_ack: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgChannelCloseInit {
    pub channel_id: ChannelId,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgChannelCloseConfirm {
    pub channel_id: ChannelId,
    pub proof_init: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgPacketRecv {
    pub packets: Array<Packet>,
    pub relayer_msgs: Array<ByteArray>,
    pub relayer: ContractAddress,
    pub proof: ByteArray,
    pub proof_height: u64,
}

#[derive(Drop, Serde)]
pub struct MsgPacketAcknowledgement {
    pub packets: Array<Packet>,
    pub acknowledgements: Array<ByteArray>,
    pub proof: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgPacketTimeout {
    pub packet: Packet,
    pub proof: ByteArray,
    pub proof_height: u64,
    pub relayer: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgIntentPacketRecv {
    pub packets: Array<Packet>,
    pub market_maker_msgs: Array<ByteArray>,
    pub market_maker: ContractAddress,
}

#[derive(Drop, Serde)]
pub struct MsgBatchSend {
    pub packets: Array<Packet>,
}

#[derive(Drop, Serde)]
pub struct MsgBatchAcks {
    pub packets: Array<Packet>,
    pub acks: Array<ByteArray>,
}

#[derive(Drop, Serde)]
pub struct MsgWriteAcknowledgement {
    pub packet: Packet,
    pub acknowledgement: ByteArray,
}

#[derive(Drop, Serde)]
pub struct MsgSendPacket {
    pub source_channel_id: ChannelId,
    pub timeout_timestamp: Timestamp,
    pub data: ByteArray,
}

#[derive(Drop, Serde)]
pub struct MsgCommitMembershipProof {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub proof: ByteArray,
    pub path: ByteArray,
    pub value: ByteArray,
}

#[derive(Drop, Serde)]
pub struct MsgCommitNonMembershipProof {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub proof: ByteArray,
    pub path: ByteArray,
}
