use core::str::FromStr;

use macros::model;

// #[cfg(feature = "ethabi")]
// use crate::ibc::core::connection::counterparty::TryFromEthAbiConnectionCounterpartyError;
use crate::{
    errors::{required, MissingField, UnknownEnumVariant},
    ibc::core::connection::{
        counterparty::{Counterparty, TryFromConnectionCounterpartyError},
        state::State,
        version::Version,
    },
    id::ClientId,
};

#[model(
    proto(raw(protos::ibc::core::connection::v1::ConnectionEnd), into, from),
    // ethabi(raw(contracts::glue::IbcCoreConnectionV1ConnectionEndData), into, from)
)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct ConnectionEnd {
    pub client_id: ClientId,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty,
    pub delay_period: u64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromConnectionEndError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid client_id")]
    ClientId(#[from] <ClientId as FromStr>::Err),
    #[error("invalid version")]
    Version(#[from] UnknownEnumVariant<String>),
    #[error("invalid state")]
    State(#[from] UnknownEnumVariant<i32>),
    #[error("invalid counterparty")]
    Counterparty(#[from] TryFromConnectionCounterpartyError),
}

impl TryFrom<protos::ibc::core::connection::v1::ConnectionEnd> for ConnectionEnd {
    type Error = TryFromConnectionEndError;

    fn try_from(
        val: protos::ibc::core::connection::v1::ConnectionEnd,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val
                .client_id
                .parse()
                .map_err(TryFromConnectionEndError::ClientId)?,
            versions: val
                .versions
                .into_iter()
                .map(|x| x.try_into().map_err(TryFromConnectionEndError::Version))
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromConnectionEndError::State)?,
            counterparty: required!(val.counterparty)?
                .try_into()
                .map_err(TryFromConnectionEndError::Counterparty)?,
            delay_period: val.delay_period,
        })
    }
}

impl From<ConnectionEnd> for protos::ibc::core::connection::v1::ConnectionEnd {
    fn from(val: ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id.to_string(),
            versions: val.versions.into_iter().map(Into::into).collect(),
            state: val.state as i32,
            counterparty: Some(val.counterparty.into()),
            delay_period: val.delay_period,
        }
    }
}

// #[derive(Debug)]
// #[cfg(feature = "ethabi")]
// pub enum TryFromEthAbiConnectionEndError {
//     ClientId(<ClientId as FromStr>::Err),
//     Version(UnknownEnumVariant<String>),
//     State(UnknownEnumVariant<u8>),
//     Counterparty(TryFromEthAbiConnectionCounterpartyError),
// }

// #[cfg(feature = "ethabi")]
// impl TryFrom<contracts::glue::IbcCoreConnectionV1ConnectionEndData> for ConnectionEnd {
//     type Error = TryFromEthAbiConnectionEndError;

//     fn try_from(
//         val: contracts::glue::IbcCoreConnectionV1ConnectionEndData,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self {
//             client_id: val
//                 .client_id
//                 .parse()
//                 .map_err(TryFromEthAbiConnectionEndError::ClientId)?,
//             versions: val
//                 .versions
//                 .into_iter()
//                 .map(|x| {
//                     x.try_into()
//                         .map_err(TryFromEthAbiConnectionEndError::Version)
//                 })
//                 .collect::<Result<_, _>>()?,
//             state: val
//                 .state
//                 .try_into()
//                 .map_err(TryFromEthAbiConnectionEndError::State)?,
//             counterparty: val
//                 .counterparty
//                 .try_into()
//                 .map_err(TryFromEthAbiConnectionEndError::Counterparty)?,
//             delay_period: val.delay_period,
//         })
//     }
// }

// #[cfg(feature = "ethabi")]
// impl From<ConnectionEnd> for contracts::glue::IbcCoreConnectionV1ConnectionEndData {
//     fn from(val: ConnectionEnd) -> Self {
//         Self {
//             client_id: val.client_id.to_string(),
//             versions: val.versions.into_iter().map(Into::into).collect(),
//             state: val.state.into(),
//             counterparty: val.counterparty.into(),
//             delay_period: val.delay_period,
//         }
//     }
// }

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;

    use super::*;
    use crate::{
        encoding::{Encode, EthAbi},
        id::ConnectionId,
    };

    alloy::sol! {
        struct SolIBCConnection {
            SolIBCConnectionState state;
            uint32 clientId;
            uint32 counterpartyClientId;
            uint32 counterpartyConnectionId;
        }

        enum SolIBCConnectionState {
            Unspecified,
            Init,
            TryOpen,
            Open
        }


    }

    impl Encode<EthAbi> for ConnectionEnd {
        fn encode(self) -> Vec<u8> {
            SolIBCConnection {
                state: self.state.into(),
                clientId: self.client_id.id(),
                counterpartyClientId: self.counterparty.client_id.id(),
                counterpartyConnectionId: self
                    .counterparty
                    .connection_id
                    .unwrap_or(ConnectionId::new(0))
                    .id(),
            }
            .abi_encode()
        }
    }

    impl From<State> for SolIBCConnectionState {
        fn from(value: State) -> SolIBCConnectionState {
            match value {
                State::UninitializedUnspecified => SolIBCConnectionState::Unspecified,
                State::Init => SolIBCConnectionState::Init,
                State::Tryopen => SolIBCConnectionState::TryOpen,
                State::Open => SolIBCConnectionState::Open,
            }
        }
    }
}
