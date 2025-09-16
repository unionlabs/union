use ibc_union_spec::datagram::{MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout};
use jsonrpsee::core::traits::ToRpcParams;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Call {
    OnRecvPacket(MsgPacketRecv),
    OnAcknowledgePacket(MsgPacketAcknowledgement),
    OnTimeoutPacket(MsgPacketTimeout),
}
