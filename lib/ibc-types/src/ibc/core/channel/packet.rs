use crate::{errors::MissingField, ibc::core::client::height::Height};

#[derive(Debug, Clone)]
pub struct Packet {
    pub sequence: u64,
    pub source_port: String,
    pub source_channel: String,
    pub destination_port: String,
    pub destination_channel: String,
    pub data: Vec<u8>,
    pub timeout_height: Height,
    pub timeout_timestamp: u64,
}

impl From<Packet> for protos::ibc::core::channel::v1::Packet {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel,
            destination_port: value.destination_port,
            destination_channel: value.destination_channel,
            data: value.data,
            timeout_height: Some(value.timeout_height.into()),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}

impl TryFrom<protos::ibc::core::channel::v1::Packet> for Packet {
    type Error = MissingField;

    fn try_from(proto: protos::ibc::core::channel::v1::Packet) -> Result<Self, Self::Error> {
        Ok(Packet {
            sequence: proto.sequence,
            source_port: proto.source_port,
            source_channel: proto.source_channel,
            destination_port: proto.destination_port,
            destination_channel: proto.destination_channel,
            data: proto.data,
            timeout_height: proto
                .timeout_height
                .ok_or(MissingField("timeout_height"))?
                .into(),
            timeout_timestamp: proto.timeout_timestamp,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Packet> for contracts::ibc_handler::IbcCoreChannelV1PacketData {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel,
            destination_port: value.destination_port,
            destination_channel: value.destination_channel,
            data: value.data.into(),
            timeout_height: value.timeout_height.into(),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}
