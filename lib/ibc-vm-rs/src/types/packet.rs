use serde::{Deserialize, Serialize};
use unionlabs::id::ChannelId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Packet {
    pub sequence: u32,
    pub source_channel: ChannelId,
    pub destination_channel: ChannelId,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}
