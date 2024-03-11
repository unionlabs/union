use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField, UnknownEnumVariant},
    ibc::core::channel::{
        counterparty::{Counterparty, TryFromChannelCounterpartyError},
        order::Order,
        state::State,
    },
    id::{ConnectionId, ConnectionIdValidator},
    validated::{Validate, ValidateT},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::core::channel::v1::Channel, into, from)]
pub struct Channel {
    pub state: State,
    pub ordering: Order,
    pub counterparty: Counterparty,
    pub connection_hops: Vec<ConnectionId>,
    // REVIEW(benluelo): Make this more strongly typed?
    pub version: String,
}

impl From<Channel> for protos::ibc::core::channel::v1::Channel {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty.into()),
            connection_hops: value
                .connection_hops
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            version: value.version,
        }
    }
}

#[derive(Debug)]
pub enum TryFromChannelError {
    MissingField(MissingField),
    State(UnknownEnumVariant<i32>),
    Counterparty(TryFromChannelCounterpartyError),
    Ordering(UnknownEnumVariant<i32>),
    ConnectionHops(<ConnectionIdValidator as Validate<String>>::Error),
}

impl TryFrom<protos::ibc::core::channel::v1::Channel> for Channel {
    type Error = TryFromChannelError;

    fn try_from(proto: protos::ibc::core::channel::v1::Channel) -> Result<Self, Self::Error> {
        Ok(Channel {
            state: State::try_from(proto.state).map_err(TryFromChannelError::State)?,
            ordering: Order::try_from(proto.ordering).map_err(TryFromChannelError::State)?,
            counterparty: required!(proto.counterparty)?
                .try_into()
                .map_err(TryFromChannelError::Counterparty)?,
            connection_hops: proto
                .connection_hops
                .into_iter()
                .map(ValidateT::validate)
                .collect::<Result<_, _>>()
                .map_err(TryFromChannelError::ConnectionHops)?,
            version: proto.version,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Channel> for contracts::ibc_handler::IbcCoreChannelV1ChannelData {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as u8,
            ordering: value.ordering as u8,
            counterparty: value.counterparty.into(),
            connection_hops: value
                .connection_hops
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            version: value.version,
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiChannelError {
    State(UnknownEnumVariant<u8>),
    Ordering(UnknownEnumVariant<u8>),
    Counterparty(crate::TryFromEthAbiErrorOf<Counterparty>),
    ConnectionHops(<ConnectionIdValidator as Validate<String>>::Error),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::ibc_handler::IbcCoreChannelV1ChannelData> for Channel {
    type Error = TryFromEthAbiChannelError;

    fn try_from(
        value: contracts::ibc_handler::IbcCoreChannelV1ChannelData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            state: value
                .state
                .try_into()
                .map_err(TryFromEthAbiChannelError::State)?,
            ordering: value
                .ordering
                .try_into()
                .map_err(TryFromEthAbiChannelError::Ordering)?,
            counterparty: value
                .counterparty
                .try_into()
                .map_err(TryFromEthAbiChannelError::Counterparty)?,
            connection_hops: value
                .connection_hops
                .into_iter()
                .map(ValidateT::validate)
                .collect::<Result<_, _>>()
                .map_err(TryFromEthAbiChannelError::ConnectionHops)?,
            version: value.version,
        })
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Channel {
    type EthAbi = contracts::ibc_handler::IbcCoreChannelV1ChannelData;
}
