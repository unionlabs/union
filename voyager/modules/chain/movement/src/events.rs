use aptos_move_ibc::ibc;
use enumorph::Enumorph;
use voyager_message::macros::model;

#[model]
#[derive(Enumorph)]
pub enum IbcEvent {
    CreateClient(ibc::ClientCreatedEvent),
    UpdateClient(ibc::ClientUpdated),
    ConnectionOpenInit(ibc::ConnectionOpenInit),
    ConnectionOpenTry(ibc::ConnectionOpenTry),
    ConnectionOpenAck(ibc::ConnectionOpenAck),
    ConnectionOpenConfirm(ibc::ConnectionOpenConfirm),
    ChannelOpenInit(ibc::ChannelOpenInit),
    ChannelOpenTry(ibc::ChannelOpenTry),
    ChannelOpenAck(ibc::ChannelOpenAck),
    ChannelOpenConfirm(ibc::ChannelOpenConfirm),
    WriteAcknowledgement(ibc::WriteAcknowledgement),
    RecvPacket(ibc::RecvPacket),
    SendPacket(ibc::SendPacket),
    AcknowledgePacket(ibc::AcknowledgePacket),
    TimeoutPacket(ibc::TimeoutPacket),
}
