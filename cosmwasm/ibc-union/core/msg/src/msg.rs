use ibc_union_spec::{Channel, ChannelId, ClientId, ConnectionId, Packet};
use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitMsg {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgRegisterClient {
    pub client_type: String,
    pub client_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterClient(MsgRegisterClient),
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
    PacketAck(MsgPacketAcknowledgement),
    PacketTimeout(MsgPacketTimeout),
    IntentPacketRecv(MsgIntentPacketRecv),
    BatchSend(MsgBatchSend),
    BatchAcks(MsgBatchAcks),
    PacketSend(MsgSendPacket),
    WriteAcknowledgement(MsgWriteAcknowledgement),
    MigrateState(MsgMigrateState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgMigrateState {
    pub client_id: ClientId,
    pub client_state: Bytes,
    pub consensus_state: Bytes,
    pub height: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgCreateClient {
    pub client_type: String,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Bytes,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenInit {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenTry {
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
    pub client_id: ClientId,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub proof_try: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenInit {
    pub port_id: String,
    pub counterparty_port_id: Bytes,
    pub connection_id: ConnectionId,
    pub version: String,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenTry {
    pub port_id: String,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenAck {
    pub channel_id: ChannelId,
    pub counterparty_version: String,
    pub counterparty_channel_id: ChannelId,
    pub proof_try: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenConfirm {
    pub channel_id: ChannelId,
    pub proof_ack: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelCloseInit {
    pub channel_id: ChannelId,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelCloseConfirm {
    pub channel_id: ChannelId,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketRecv {
    pub packets: Vec<Packet>,
    pub relayer_msgs: Vec<Bytes>,
    pub relayer: String,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketAcknowledgement {
    pub packets: Vec<Packet>,
    pub acknowledgements: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketTimeout {
    pub packet: Packet,
    pub proof: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgIntentPacketRecv {
    pub packets: Vec<Packet>,
    pub market_maker_msgs: Vec<Bytes>,
    pub market_maker: String,
    pub empty_proof: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgBatchSend {
    pub packets: Vec<Packet>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgBatchAcks {
    pub packets: Vec<Packet>,
    pub acks: Vec<Bytes>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgWriteAcknowledgement {
    pub channel_id: ChannelId,
    pub packet: Packet,
    pub acknowledgement: Bytes,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendPacket {
    pub source_channel_id: ChannelId,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub data: Bytes,
}
