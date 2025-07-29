use enumorph::Enumorph;
use macros::model;
use sui_light_client_types::U64;
use sui_sdk::types::base_types::SuiAddress;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateClient {
    pub client_id: u32,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateClient {
    pub client_id: u32,
    pub height: U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectionOpenInit {
    pub connection_id: u32,
    pub client_id: u32,
    pub counterparty_client_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectionOpenTry {
    pub connection_id: u32,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectionOpenAck {
    pub connection_id: u32,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectionOpenConfirm {
    pub connection_id: u32,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
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
    pub counterparty_version: String,
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Packet {
    pub source_channel_id: u32,
    pub destination_channel_id: u32,
    pub data: Vec<u8>,
    pub timeout_height: U64,
    pub timeout_timestamp: U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketSend {
    pub channel_id: u32,
    pub packet_hash: Vec<u8>,

    pub packet: Packet,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketRecv {
    pub channel_id: u32,
    pub packet_hash: Vec<u8>,
    pub maker: SuiAddress,
    pub maker_msg: Vec<u8>,
}

#[model]
#[derive(Enumorph)]
pub enum IbcEvent {
    CreateClient(CreateClient),
    UpdateClient(UpdateClient),
    ConnectionOpenInit(ConnectionOpenInit),
    ConnectionOpenTry(ConnectionOpenTry),
    ConnectionOpenAck(ConnectionOpenAck),
    ConnectionOpenConfirm(ConnectionOpenConfirm),
    ChannelOpenInit(ChannelOpenInit),
    ChannelOpenTry(ChannelOpenTry),
    ChannelOpenAck(ChannelOpenAck),
    ChannelOpenConfirm(ChannelOpenConfirm),
    // WriteAcknowledgement(ibc::WriteAck),
    // RecvPacket(ibc::PacketRecv),
    PacketSend(PacketSend),
    PacketRecv(PacketRecv),
    // AcknowledgePacket(ibc::PacketAck),
    // TimeoutPacket(ibc::TimeoutPacket),
}
