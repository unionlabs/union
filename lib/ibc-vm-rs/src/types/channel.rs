use serde::{Deserialize, Serialize};
use unionlabs::id::{ChannelId, ConnectionId, PortId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChannelState {
    Unspecified,
    Init,
    TryOpen,
    Open,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChannelOrder {
    Unspecified,
    Unordered,
    Ordered,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub state: ChannelState,
    pub ordering: ChannelOrder,
    pub connection_id: ConnectionId,
    pub counterparty_channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub version: String,
}
