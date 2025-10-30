use ibc_union_spec::{Channel, ChannelId, ClientId, ConnectionId, Packet, Timestamp};
use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitMsg {
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgRegisterClient {
    pub client_type: String,
    pub client_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "cw-orch-interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    RegisterClient(MsgRegisterClient),
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    ForceUpdateClient(MsgForceUpdateClient),

    ConnectionOpenInit(MsgConnectionOpenInit),
    ConnectionOpenTry(MsgConnectionOpenTry),
    ForceConnectionOpenTry(MsgConnectionOpenTry),
    ConnectionOpenAck(MsgConnectionOpenAck),
    ForceConnectionOpenAck(MsgConnectionOpenAck),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm),
    ForceConnectionOpenConfirm(MsgConnectionOpenConfirm),

    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ForceChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ForceChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),
    ForceChannelOpenConfirm(MsgChannelOpenConfirm),
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

    CommitMembershipProof(MsgCommitMembershipProof),
    CommitNonMembershipProof(MsgCommitNonMembershipProof),

    #[serde(untagged)]
    AccessManaged(access_managed::ExecuteMsg),

    #[serde(untagged)]
    Upgradable(upgradable::msg::ExecuteMsg),
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
pub struct MsgForceUpdateClient {
    pub client_id: ClientId,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenInit {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenTry {
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
    pub client_id: ClientId,
    pub proof_init: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub proof_try: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: Bytes,
    pub proof_height: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgWriteAcknowledgement {
    pub packet: Packet,
    pub acknowledgement: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendPacket {
    pub source_channel_id: ChannelId,
    pub timeout_timestamp: Timestamp,
    pub data: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgCommitMembershipProof {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub proof: Bytes,
    pub path: Bytes,
    pub value: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgCommitNonMembershipProof {
    pub client_id: ClientId,
    pub proof_height: u64,
    pub proof: Bytes,
    pub path: Bytes,
}
