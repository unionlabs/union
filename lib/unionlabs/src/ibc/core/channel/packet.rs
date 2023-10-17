use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ibc::core::client::height::Height,
    id::{ChannelId, ChannelIdValidator},
    validated::{Validate, ValidateT},
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Packet {
    pub sequence: u64,
    pub source_port: String,
    pub source_channel: ChannelId,
    pub destination_port: String,
    pub destination_channel: ChannelId,
    #[serde(with = "::serde_utils::hex_string")]
    pub data: Vec<u8>,
    pub timeout_height: Height,
    pub timeout_timestamp: u64,
}

impl Proto for Packet {
    type Proto = protos::ibc::core::channel::v1::Packet;
}

impl TypeUrl for protos::ibc::core::channel::v1::Packet {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.Packet";
}

impl From<Packet> for protos::ibc::core::channel::v1::Packet {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel.to_string(),
            destination_port: value.destination_port,
            destination_channel: value.destination_channel.to_string(),
            data: value.data,
            timeout_height: Some(value.timeout_height.into()),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}

#[derive(Debug)]
pub enum TryFromPacketError {
    MissingField(MissingField),
    SourceChannel(<ChannelIdValidator as Validate<String>>::Error),
    DestinationChannel(<ChannelIdValidator as Validate<String>>::Error),
}

impl TryFrom<protos::ibc::core::channel::v1::Packet> for Packet {
    type Error = TryFromPacketError;

    fn try_from(proto: protos::ibc::core::channel::v1::Packet) -> Result<Self, Self::Error> {
        Ok(Packet {
            sequence: proto.sequence,
            source_port: proto.source_port,
            source_channel: proto
                .source_channel
                .validate()
                .map_err(TryFromPacketError::SourceChannel)?,
            destination_port: proto.destination_port,
            destination_channel: proto
                .destination_channel
                .validate()
                .map_err(TryFromPacketError::DestinationChannel)?,
            data: proto.data,
            timeout_height: required!(proto.timeout_height)?.into(),
            timeout_timestamp: proto.timeout_timestamp,
        })
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiPacketError {
    SourceChannel(<ChannelIdValidator as Validate<String>>::Error),
    DestinationChannel(<ChannelIdValidator as Validate<String>>::Error),
}

#[cfg(feature = "ethabi")]
impl From<Packet> for contracts::ibc_handler::IbcCoreChannelV1PacketData {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel.to_string(),
            destination_port: value.destination_port,
            destination_channel: value.destination_channel.to_string(),
            data: value.data.into(),
            timeout_height: value.timeout_height.into(),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}
