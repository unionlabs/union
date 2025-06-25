use alloy::primitives::FixedBytes;
use ibc_union_spec::{event::PacketSend, event::CreateClient, ChannelId, ConnectionId, ClientId, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionConfirm {
    pub connection_id: u32,
    pub counterparty_connection_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelOpenConfirm {
    pub channel_id: u32,
    pub counterparty_channel_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateClientConfirm {
    pub client_id: u32
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketRecv {
    pub hash: FixedBytes<32>
}