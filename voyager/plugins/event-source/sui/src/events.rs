use enumorph::Enumorph;
use macros::model;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateClient {
    pub client_id: u32,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

#[model]
#[derive(Enumorph)]
pub enum IbcEvent {
    CreateClient(CreateClient),
    // UpdateClient(ibc::UpdateClient),
    // ConnectionOpenInit(ibc::ConnectionOpenInit),
    // ConnectionOpenTry(ibc::ConnectionOpenTry),
    // ConnectionOpenAck(ibc::ConnectionOpenAck),
    // ConnectionOpenConfirm(ibc::ConnectionOpenConfirm),
    // ChannelOpenInit(ibc::ChannelOpenInit),
    // ChannelOpenTry(ibc::ChannelOpenTry),
    // ChannelOpenAck(ibc::ChannelOpenAck),
    // ChannelOpenConfirm(ibc::ChannelOpenConfirm),
    // WriteAcknowledgement(ibc::WriteAck),
    // RecvPacket(ibc::PacketRecv),
    // SendPacket(ibc::PacketSend),
    // AcknowledgePacket(ibc::PacketAck),
    // TimeoutPacket(ibc::TimeoutPacket),
}
