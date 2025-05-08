use enumorph::Enumorph;
use macros::model;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateClient {
    pub client_id: u32,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientUpdated {
    pub client_id: u32,
    pub client_type: String,
    pub height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectionOpenInit {
    pub connection_id: u32,
    pub client_id: u32,
    pub counterparty_client_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelOpenInit {
    pub port_id: String,
    pub channel_id: u32,
    pub counterparty_port_id: Vec<u8>,
    pub connection_id: u32,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelOpenTry {
    pub port_id: String,
    pub channel_id: u32,
    pub counterparty_port_id: Vec<u8>,
    pub counterparty_channel_id: u32,
    pub connection_id: u32,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelOpenAck {
    pub port_id: String,
    pub channel_id: u32,
    pub counterparty_port_id: Vec<u8>,
    pub counterparty_channel_id: u32,
    pub connection_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChannelOpenConfirm {
    pub port_id: String,
    pub channel_id: u32,
    pub counterparty_port_id: Vec<u8>,
    pub counterparty_channel_id: u32,
    pub connection_id: u32,
}

#[model]
#[derive(Enumorph)]
pub enum IbcEvent {
    CreateClient(CreateClient),
    // UpdateClient(UpdateClient),
    ConnectionOpenInit(ConnectionOpenInit),
    // ConnectionOpenTry(ConnectionOpenTry),
    // ConnectionOpenAck(ConnectionOpenAck),
    // ConnectionOpenConfirm(ConnectionOpenConfirm),
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
