use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, IbcMsg, IbcTimeout, Timestamp};
use ethabi::{ParamType, Token};
use ibc_solidity::ibc::{
    MsgBatchAcks, MsgBatchSend, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck,
    MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient,
    MsgIntentPacketRecv, MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout,
    MsgUpdateClient,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InitMsg {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MsgRegisterClient {
    pub client_type: String,
    pub client_address: Addr,
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
}
