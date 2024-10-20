use alloy::sol_types::SolValue;
use macros::model;

use crate::{
    encoding::{Encode, EthAbi},
    errors::{required, MissingField, UnknownEnumVariant},
    ibc::core::channel::{
        counterparty::{Counterparty, TryFromChannelCounterpartyError},
        order::Order,
        state::State,
    },
    id::{ConnectionId, ConnectionIdValidator},
    validated::{Validate, ValidateT},
};

#[model(proto(raw(protos::ibc::core::channel::v1::Channel), into, from))]
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

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromChannelError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid state")]
    State(#[source] UnknownEnumVariant<i32>),
    #[error("invalid counterparty")]
    Counterparty(#[from] TryFromChannelCounterpartyError),
    #[error("invalid ordering")]
    Ordering(#[source] UnknownEnumVariant<i32>),
    #[error("invalid connection_hops")]
    ConnectionHops(#[from] <ConnectionIdValidator as Validate<String>>::Error),
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

alloy::sol! {
    enum SolIBCChannelState {
        Unspecified,
        Init,
        TryOpen,
        Open,
        Closed
    }

    enum SolIBCChannelOrder {
        Unspecified,
        Unordered,
        Ordered
    }

    struct SolIBCChannel {
        SolIBCChannelState state;
        SolIBCChannelOrder ordering;
        uint32 connectionId;
        uint32 counterpartyChannelId;
        string counterpartyPortId;
        string version;
    }
}

impl Encode<EthAbi> for Channel {
    fn encode(self) -> Vec<u8> {
        SolIBCChannel {
            state: self.state.into(),
            ordering: self.ordering.into(),
            connectionId: self.connection_hops[0]
                .strip_suffix(char::is_numeric)
                .unwrap()
                .parse()
                .unwrap(),
            counterpartyChannelId: self
                .counterparty
                .channel_id
                .strip_suffix(char::is_numeric)
                .unwrap_or("0")
                .parse()
                .unwrap(),
            counterpartyPortId: self
                .counterparty
                .port_id
                .strip_suffix(char::is_numeric)
                .unwrap()
                .parse()
                .unwrap(),
            version: self.version,
        }
        .abi_encode()
    }
}

impl From<State> for SolIBCChannelState {
    fn from(value: State) -> Self {
        match value {
            State::UninitializedUnspecified => SolIBCChannelState::Unspecified,
            State::Init => SolIBCChannelState::Init,
            State::Tryopen => SolIBCChannelState::TryOpen,
            State::Open => SolIBCChannelState::Open,
            State::Closed => SolIBCChannelState::Closed,
        }
    }
}

impl From<Order> for SolIBCChannelOrder {
    fn from(value: Order) -> SolIBCChannelOrder {
        match value {
            Order::NoneUnspecified => SolIBCChannelOrder::Unspecified,
            Order::Unordered => SolIBCChannelOrder::Unordered,
            Order::Ordered => SolIBCChannelOrder::Ordered,
        }
    }
}
