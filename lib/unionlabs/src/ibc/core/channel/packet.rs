use core::num::{NonZeroU64, TryFromIntError};

use macros::model;

use crate::{
    errors::{MissingField, required},
    ibc::core::client::height::Height,
    id::{ChannelId, Ics24IdParseError, ParsePrefixedIdError, PortId},
    primitives::Bytes,
};

#[model(proto(raw(protos::ibc::core::channel::v1::Packet), into, from))]
pub struct Packet {
    pub sequence: NonZeroU64,
    pub source_port: PortId,
    pub source_channel: ChannelId,
    pub destination_port: PortId,
    pub destination_channel: ChannelId,
    pub data: Bytes,
    pub timeout_height: Height,
    pub timeout_timestamp: u64,
}

impl From<Packet> for protos::ibc::core::channel::v1::Packet {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence.get(),
            source_port: value.source_port.to_string(),
            source_channel: value.source_channel.to_string_prefixed(),
            destination_port: value.destination_port.to_string(),
            destination_channel: value.destination_channel.to_string_prefixed(),
            data: value.data.into(),
            timeout_height: Some(value.timeout_height.into()),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromPacketError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid sequence")]
    Sequence(#[source] TryFromIntError),
    #[error("invalid source_channel")]
    SourceChannel(#[source] ParsePrefixedIdError),
    #[error("invalid destination_channel")]
    DestinationChannel(#[source] ParsePrefixedIdError),
    #[error("invalid source_port")]
    SourcePort(#[source] Ics24IdParseError),
    #[error("invalid destination_port")]
    DestinationPort(#[source] Ics24IdParseError),
}

impl TryFrom<protos::ibc::core::channel::v1::Packet> for Packet {
    type Error = TryFromPacketError;

    fn try_from(proto: protos::ibc::core::channel::v1::Packet) -> Result<Self, Self::Error> {
        Ok(Packet {
            sequence: proto
                .sequence
                .try_into()
                .map_err(TryFromPacketError::Sequence)?,
            source_port: proto
                .source_port
                .try_into()
                .map_err(TryFromPacketError::SourcePort)?,
            source_channel: ChannelId::from_str_prefixed(&proto.source_channel)
                .map_err(TryFromPacketError::SourceChannel)?,
            destination_port: proto
                .destination_port
                .try_into()
                .map_err(TryFromPacketError::DestinationPort)?,
            destination_channel: ChannelId::from_str_prefixed(&proto.destination_channel)
                .map_err(TryFromPacketError::DestinationChannel)?,
            data: proto.data.into(),
            timeout_height: required!(proto.timeout_height)?.into(),
            timeout_timestamp: proto.timeout_timestamp,
        })
    }
}
