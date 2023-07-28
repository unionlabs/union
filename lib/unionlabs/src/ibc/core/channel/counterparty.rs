#[derive(Debug, Clone)]
pub struct Counterparty {
    pub port_id: String,
    pub channel_id: String,
}

impl From<Counterparty> for protos::ibc::core::channel::v1::Counterparty {
    fn from(value: Counterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

impl From<protos::ibc::core::channel::v1::Counterparty> for Counterparty {
    fn from(proto: protos::ibc::core::channel::v1::Counterparty) -> Self {
        Self {
            port_id: proto.port_id,
            channel_id: proto.channel_id,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Counterparty> for contracts::ibc_handler::IbcCoreChannelV1CounterpartyData {
    fn from(value: Counterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::ibc_handler::IbcCoreChannelV1CounterpartyData> for Counterparty {
    fn from(value: contracts::ibc_handler::IbcCoreChannelV1CounterpartyData) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}
