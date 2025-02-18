use aptos_move_ibc::ibc;
use enumorph::Enumorph;
use macros::model;

#[model]
#[derive(Enumorph)]
pub enum IbcEvent {
    CreateClient(ibc::CreateClient),
    UpdateClient(ibc::UpdateClient),
    ConnectionOpenInit(ibc::ConnectionOpenInit),
    ConnectionOpenTry(ibc::ConnectionOpenTry),
    ConnectionOpenAck(ibc::ConnectionOpenAck),
    ConnectionOpenConfirm(ibc::ConnectionOpenConfirm),
    ChannelOpenInit(ibc::ChannelOpenInit),
    ChannelOpenTry(ibc::ChannelOpenTry),
    ChannelOpenAck(ibc::ChannelOpenAck),
    ChannelOpenConfirm(ibc::ChannelOpenConfirm),
    WriteAcknowledgement(ibc::WriteAck),
    RecvPacket(ibc::PacketRecv),
    SendPacket(ibc::PacketSend),
    AcknowledgePacket(ibc::PacketAck),
    TimeoutPacket(ibc::TimeoutPacket),
}
