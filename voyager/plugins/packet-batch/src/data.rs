use std::time::{SystemTime, UNIX_EPOCH};

use ibc_union_spec::Packet;
use macros::model;
use unionlabs::primitives::Bytes;

#[model]
pub enum ModuleData {
    BatchSendPacket(Vec<BatchSend>),
    BatchAckPacket(Vec<BatchAck>),
}

#[model]
pub struct BatchSend {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    pub packet: Packet,
}

impl BatchSend {
    pub fn new(packet: Packet) -> Self {
        Self {
            first_seen_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .expect("how many milliseconds can there be man"),
            packet,
        }
    }
}

#[model]
pub struct BatchAck {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    pub packet: Packet,
    pub ack: Bytes,
}

impl BatchAck {
    pub fn new(packet: Packet, ack: Bytes) -> Self {
        Self {
            first_seen_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .expect("how many milliseconds can there be man"),
            packet,
            ack,
        }
    }
}
