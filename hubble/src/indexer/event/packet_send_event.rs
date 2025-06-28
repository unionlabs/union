use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{BlockHeight, ChannelId, PacketData, PacketHash, TimeoutTimestamp},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PacketSendEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub timeout_height: BlockHeight,
    pub timeout_timestamp: TimeoutTimestamp,
    pub data: PacketData,
}
