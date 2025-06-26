use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ChannelId, Maker, MakerMsg, PacketHash},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PacketRecvEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub maker: Maker,
    pub maker_msg: MakerMsg,
}
