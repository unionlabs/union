use macros::model;

use crate::{
    ibc::core::channel::{counterparty::Counterparty, order::Order, state::State},
    id::ConnectionId,
};

#[model(proto(raw(protos::ibc::core::channel::v1::Channel), into, from))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct Channel {
    pub state: State,
    pub ordering: Order,
    pub counterparty: Counterparty,
    // TODO: this must be non-empty
    pub connection_hops: Vec<ConnectionId>,
    // REVIEW(benluelo): Make this more strongly typed?
    pub version: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField, UnknownEnumVariant},
        ibc::core::channel::{
            channel::Channel, counterparty::proto::TryFromChannelCounterpartyError, order::Order,
            state::State,
        },
        id::{ConnectionId, ParsePrefixedIdError},
    };

    impl From<Channel> for protos::ibc::core::channel::v1::Channel {
        fn from(value: Channel) -> Self {
            Self {
                state: value.state as i32,
                ordering: value.ordering as i32,
                counterparty: Some(value.counterparty.into()),
                connection_hops: value
                    .connection_hops
                    .into_iter()
                    .map(|x| x.to_string_prefixed())
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
        ConnectionHops(#[from] ParsePrefixedIdError),
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
                    .map(|s| ConnectionId::parse_prefixed(&s))
                    .collect::<Result<_, _>>()
                    .map_err(TryFromChannelError::ConnectionHops)?,
                version: proto.version,
            })
        }
    }
}
