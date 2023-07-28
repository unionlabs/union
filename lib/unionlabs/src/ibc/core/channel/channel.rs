use crate::{
    errors::MissingField,
    ibc::core::channel::{counterparty::Counterparty, order::Order, state::State},
    IntoProto, TryFromProto, TypeUrl,
};

#[derive(Debug, Clone)]
pub struct Channel {
    pub state: State,
    pub ordering: Order,
    pub counterparty: Counterparty,
    pub connection_hops: Vec<String>,
    // REVIEW(benluelo): Make this more strongly typed?
    pub version: String,
}

impl From<Channel> for protos::ibc::core::channel::v1::Channel {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty.into()),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

impl TryFrom<protos::ibc::core::channel::v1::Channel> for Channel {
    type Error = MissingField;

    fn try_from(proto: protos::ibc::core::channel::v1::Channel) -> Result<Self, Self::Error> {
        Ok(Channel {
            state: State::try_from(proto.state).unwrap(),
            ordering: Order::try_from(proto.ordering).unwrap(),
            counterparty: proto
                .counterparty
                .ok_or(MissingField("counterparty"))?
                .into(),
            connection_hops: proto.connection_hops,
            version: proto.version,
        })
    }
}

impl IntoProto for Channel {
    type Proto = protos::ibc::core::channel::v1::Channel;
}

impl TryFromProto for Channel {
    type Proto = protos::ibc::core::channel::v1::Channel;
}

impl TypeUrl for protos::ibc::core::channel::v1::Channel {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.ChannelEnd";
}

#[cfg(feature = "ethabi")]
impl From<Channel> for contracts::ibc_handler::IbcCoreChannelV1ChannelData {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as u8,
            ordering: value.ordering as u8,
            counterparty: value.counterparty.into(),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::ibc_handler::IbcCoreChannelV1ChannelData> for Channel {
    type Error = crate::errors::UnknownEnumVariant<u8>;

    fn try_from(
        value: contracts::ibc_handler::IbcCoreChannelV1ChannelData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            state: value.state.try_into()?,
            ordering: value.ordering.try_into()?,
            counterparty: value.counterparty.into(),
            connection_hops: value.connection_hops,
            version: value.version,
        })
    }
}
