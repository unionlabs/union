use ibc_union_spec::{event::PacketSend, event::CreateClient, ChannelId, ConnectionId, ClientId, Timestamp};

pub struct ConnectionConfirm {
    pub connection_id: u32,
    pub counterparty_connection_id: u32,
}

pub struct ChannelOpenConfirm {
    pub channel_id: u32,
    pub counterparty_channel_id: u32,
}

pub struct CreateClientConfirm {
    pub client_id: u32
}

// pub struct PacketRecv {
//     pub hash: u32
// }