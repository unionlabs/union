use enumorph::Enumorph;
use macros::model;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateClient {
    pub client_id: u32,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

struct ClientUpdated {
    client_id: u32,
    client_type: String,
    height: u64,
}

struct ConnectionOpenInit {
    connection_id: u32,
    client_id: u32,
    counterparty_client_id: u32,
}

struct ChannelOpenInit {
    port_id: String,
    channel_id: u32,
    counterparty_port_id: Vec<u8>,
    connection_id: u32,
    version: String,
}

struct ChannelOpenTry {
    port_id: String,
    channel_id: u32,
    counterparty_port_id: Vec<u8>,
    counterparty_channel_id: u32,
    connection_id: u32,
    version: String,
}

struct ChannelOpenAck {
    port_id: String,
    channel_id: u32,
    counterparty_port_id: Vec<u8>,
    counterparty_channel_id: u32,
    connection_id: u32,
}

struct ChannelOpenConfirm {
    port_id: String,
    channel_id: u32,
    counterparty_port_id: Vec<u8>,
    counterparty_channel_id: u32,
    connection_id: u32,
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
