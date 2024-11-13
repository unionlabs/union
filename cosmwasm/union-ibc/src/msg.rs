use cosmwasm_std::Binary;
use ibc_solidity::cosmwasm::types::ibc::{
    MsgBatchAcks, MsgBatchSend, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck,
    MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient,
    MsgIntentPacketRecv, MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout,
    MsgUpdateClient, Packet,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InitMsg {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MsgRegisterClient {
    pub client_type: String,
    pub client_address: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MsgWriteAcknowledgement {
    pub channel_id: u32,
    pub packet: Packet,
    pub acknowledgement: Binary,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MsgSendPacket {
    pub source_channel: u32,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub data: Binary,
}
