use ibc_solidity::Packet;
use unionlabs_primitives::Bytes;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitMsg {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgRegisterClient {
    pub client_type: String,
    pub client_address: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgCreateClient {
    pub client_type: String,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgUpdateClient {
    pub client_id: u32,
    pub client_message: Bytes,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenInit {
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenTry {
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
    pub client_id: u32,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenAck {
    pub connection_id: u32,
    pub counterparty_connection_id: u32,
    pub proof_try: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: u32,
    pub proof_ack: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenInit {
    pub port_id: String,
    pub counterparty_port_id: Bytes,
    pub connection_id: u32,
    pub version: String,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenTry {
    pub port_id: String,
    pub channel: ibc_solidity::Channel,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenAck {
    pub channel_id: u32,
    pub counterparty_version: String,
    pub counterparty_channel_id: u32,
    pub proof_try: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenConfirm {
    pub channel_id: u32,
    pub proof_ack: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelCloseInit {
    pub channel_id: u32,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelCloseConfirm {
    pub channel_id: u32,
    pub proof_init: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketRecv {
    pub packets: Vec<Packet>,
    pub relayer_msgs: Vec<Bytes>,
    pub relayer: String,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketAcknowledgement {
    pub packets: Vec<Packet>,
    pub acknowledgements: Vec<Bytes>,
    pub proof: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgPacketTimeout {
    pub packet: Packet,
    pub proof: Bytes,
    pub proof_height: u64,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgIntentPacketRecv {
    pub packets: Vec<Packet>,
    pub market_maker_msgs: Vec<Bytes>,
    pub market_maker: String,
    pub empty_proof: Bytes,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgBatchSend {
    pub source_channel: u32,
    pub packets: Vec<Packet>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgBatchAcks {
    pub source_channel: u32,
    pub packets: Vec<Packet>,
    pub acks: Vec<Bytes>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgWriteAcknowledgement {
    pub channel_id: u32,
    pub packet: Packet,
    pub acknowledgement: Bytes,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendPacket {
    pub source_channel: u32,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub data: Bytes,
}
