use macros::model;
use serde::{Deserialize, Serialize};

#[model(no_serde)]
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum IbcEvent {
    CreateClient(crate::client::ibc::ClientCreatedEvent),
    UpdateClient(crate::client::ibc::ClientUpdated),
    ConnectionOpenInit(crate::client::ibc::ConnectionOpenInit),
    ConnectionOpenTrt(crate::client::ibc::ConnectionOpenTry),
    ConnectionOpenAct(crate::client::ibc::ConnectionOpenAck),
    ConnectionOpenConfirt(crate::client::ibc::ConnectionOpenConfirm),
    ChannelOpenInit(crate::client::ibc::ChannelOpenInit),
    ChannelOpenTrt(crate::client::ibc::ChannelOpenTry),
    ChannelOpenAct(crate::client::ibc::ChannelOpenAck),
    ChannelOpenConfirt(crate::client::ibc::ChannelOpenConfirm),
    WriteAcknowledgement(crate::client::ibc::WriteAcknowledgement),
    RecvPacket(crate::client::ibc::RecvPacket),
    SendPacket(crate::client::ibc::SendPacket),
    AcknowledgePacket(crate::client::ibc::AcknowledgePacket),
    TimeoutPacket(crate::client::ibc::TimeoutPacket),
}
