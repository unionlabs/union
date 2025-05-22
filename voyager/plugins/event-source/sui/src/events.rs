use enumorph::Enumorph;
use ibc_union_spec::Packet;
use macros::model;
use sui_light_client_types::U64;
use unionlabs::ibc;
use unionlabs::{primitives::Bytes, tuple::AsTuple};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateClient {
    pub client_id: u32,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateClient {
    pub client_id: u32,
    pub client_type: String,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketMetadata {
    pub source_channel_id: u32,
    pub destination_channel_id: u32,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketSend {
    pub channel_id: u32,
    pub packet_hash: Bytes,

    pub packet: Packet,
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
    PacketSend(PacketSend)
    // AcknowledgePacket(ibc::PacketAck),
    // TimeoutPacket(ibc::TimeoutPacket),
}
