use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Acknowledgement, ChannelId, Maker, PacketHash},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PacketAckEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub acknowledgement: Acknowledgement,
    pub maker: Maker,
}
