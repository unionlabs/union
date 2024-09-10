use macros::model;
use serde::{Deserialize, Serialize};

#[model(no_serde)]
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum IbcEvent {
    CreateClient(crate::client::ibc::ClientCreatedEvent),
    UpdateClient(crate::client::ibc::ClientUpdated),
    ConnectionOpenInit(crate::client::ibc::ConnectionOpenInit),
    ConnectionOpenTry(crate::client::ibc::ConnectionOpenTry),
    ConnectionOpenAck(crate::client::ibc::ConnectionOpenAck),
    ConnectionOpenConfirm(crate::client::ibc::ConnectionOpenConfirm),
    ChannelOpenInit(crate::client::ibc::ChannelOpenInit),
    ChannelOpenTry(crate::client::ibc::ChannelOpenTry),
    ChannelOpenAck(crate::client::ibc::ChannelOpenAck),
    ChannelOpenConfirm(crate::client::ibc::ChannelOpenConfirm),
    WriteAcknowledgement(crate::client::ibc::WriteAcknowledgement),
    RecvPacket(crate::client::ibc::RecvPacket),
    SendPacket(crate::client::ibc::SendPacket),
    AcknowledgePacket(crate::client::ibc::AcknowledgePacket),
    TimeoutPacket(crate::client::ibc::TimeoutPacket),
}
