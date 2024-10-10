use core::str::FromStr;

use macros::model;

use crate::id::PortId;

#[model(proto(raw(protos::ibc::core::channel::v1::Counterparty), into, from))]
pub struct Counterparty {
    pub port_id: PortId,
    // TODO: Option<ChannelId>, same as connection counterparty
    pub channel_id: String,
}

impl From<Counterparty> for protos::ibc::core::channel::v1::Counterparty {
    fn from(value: Counterparty) -> Self {
        Self {
            port_id: value.port_id.to_string(),
            channel_id: value.channel_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromChannelCounterpartyError {
    #[error("error parsing port id")]
    PortId(#[source] <PortId as FromStr>::Err),
}

impl TryFrom<protos::ibc::core::channel::v1::Counterparty> for Counterparty {
    type Error = TryFromChannelCounterpartyError;

    fn try_from(proto: protos::ibc::core::channel::v1::Counterparty) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: proto
                .port_id
                .parse()
                .map_err(TryFromChannelCounterpartyError::PortId)?,
            channel_id: proto.channel_id,
        })
    }
}
