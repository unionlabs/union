use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ChannelId, ConnectionId, PortId},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelOpenConfirmEvent {
    #[serde(flatten)]
    pub header: Header,
    pub connection_id: ConnectionId,
    pub channel_id: ChannelId,
    pub port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub counterparty_port_id: PortId,
}
