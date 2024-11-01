use macros::model;

// #[cfg(feature = "ethabi")]
// use crate::ibc::core::channel::counterparty::TryFromEthAbiChannelCounterpartyError;
use crate::{
    errors::{required, MissingField, UnknownEnumVariant},
    ibc::core::channel::{
        counterparty::{Counterparty, TryFromChannelCounterpartyError},
        order::Order,
        state::State,
    },
    id::{ConnectionId, ParsePrefixedIdError},
};

#[model(
    proto(raw(protos::ibc::core::channel::v1::Channel), into, from),
    // ethabi(raw(contracts::ibc_handler::IbcCoreChannelV1ChannelData), into, from)
)]
pub struct Channel {
    pub state: State,
    pub ordering: Order,
    pub counterparty: Counterparty,
    pub connection_hops: Vec<ConnectionId>,
    // REVIEW(benluelo): Make this more strongly typed?
    pub version: String,
    pub upgrade_sequence: u64,
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
                .map(|x| x.to_string_prefixed())
                .collect(),
            version: value.version,
            upgrade_sequence: value.upgrade_sequence,
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
                .map(|c| ConnectionId::from_str_prefixed(&c))
                .collect::<Result<_, _>>()
                .map_err(TryFromChannelError::ConnectionHops)?,
            version: proto.version,
            upgrade_sequence: proto.upgrade_sequence,
        })
    }
}

// #[cfg(feature = "ethabi")]
// impl From<Channel> for contracts::ibc_handler::IbcCoreChannelV1ChannelData {
//     fn from(value: Channel) -> Self {
//         Self {
//             state: value.state as u8,
//             ordering: value.ordering as u8,
//             counterparty: value.counterparty.into(),
//             connection_hops: value
//                 .connection_hops
//                 .into_iter()
//                 .map(|x| x.to_string_prefixed())
//                 .collect(),
//             version: value.version,
//         }
//     }
// }

// #[cfg(feature = "ethabi")]
// #[derive(Debug)]
// pub enum TryFromEthAbiChannelError {
//     State(UnknownEnumVariant<u8>),
//     Ordering(UnknownEnumVariant<u8>),
//     Counterparty(TryFromEthAbiChannelCounterpartyError),
//     ConnectionHops(ParsePrefixedIdError),
// }

// #[cfg(feature = "ethabi")]
// impl TryFrom<contracts::ibc_handler::IbcCoreChannelV1ChannelData> for Channel {
//     type Error = TryFromEthAbiChannelError;

//     fn try_from(
//         value: contracts::ibc_handler::IbcCoreChannelV1ChannelData,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self {
//             state: value
//                 .state
//                 .try_into()
//                 .map_err(TryFromEthAbiChannelError::State)?,
//             ordering: value
//                 .ordering
//                 .try_into()
//                 .map_err(TryFromEthAbiChannelError::Ordering)?,
//             counterparty: value
//                 .counterparty
//                 .try_into()
//                 .map_err(TryFromEthAbiChannelError::Counterparty)?,
//             connection_hops: value
//                 .connection_hops
//                 .into_iter()
//                 .map(|c| ConnectionId::from_str_prefixed(&c))
//                 .collect::<Result<_, _>>()
//                 .map_err(TryFromEthAbiChannelError::ConnectionHops)?,
//             version: value.version,
//             upgrade_sequence: 0,
//         })
//     }
// }

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;

    use super::*;
    use crate::{
        encoding::{Encode, EthAbi},
        id::ChannelId,
    };

    alloy::sol! {
        enum SolIBCChannelState {
            Unspecified,
            Init,
            TryOpen,
            Open,
            Closed,
            Flushing,
            Flushcomplete
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
                connectionId: self.connection_hops[0].id(),
                counterpartyChannelId: self
                    .counterparty
                    .channel_id
                    .unwrap_or(ChannelId::new(0))
                    .id(),
                counterpartyPortId: self
                    .counterparty
                    .port_id
                    .to_string()
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
                State::Flushing => SolIBCChannelState::Flushing,
                State::Flushcomplete => SolIBCChannelState::Flushcomplete,
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
}
