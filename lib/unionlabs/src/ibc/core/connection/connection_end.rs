use core::str::FromStr;

use frame_support_procedural::DebugNoBound;
use macros::model;

#[cfg(feature = "ethabi")]
use crate::ibc::core::connection::counterparty::TryFromEthAbiConnectionCounterpartyError;
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
    ethabi(raw(contracts::glue::IbcCoreConnectionV1ConnectionEndData), into, from)
)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct ConnectionEnd {
    pub client_id: ClientId,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty,
    pub delay_period: u64,
}

#[derive(DebugNoBound)]
pub enum TryFromConnectionEndError {
    ClientId(<ClientId as FromStr>::Err),
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<i32>),
    Counterparty(TryFromConnectionCounterpartyError),
    MissingField(MissingField),
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

#[derive(Debug)]
#[cfg(feature = "ethabi")]
pub enum TryFromEthAbiConnectionEndError {
    ClientId(<ClientId as FromStr>::Err),
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<u8>),
    Counterparty(TryFromEthAbiConnectionCounterpartyError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::IbcCoreConnectionV1ConnectionEndData> for ConnectionEnd {
    type Error = TryFromEthAbiConnectionEndError;

    fn try_from(
        val: contracts::glue::IbcCoreConnectionV1ConnectionEndData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val
                .client_id
                .parse()
                .map_err(TryFromEthAbiConnectionEndError::ClientId)?,
            versions: val
                .versions
                .into_iter()
                .map(|x| {
                    x.try_into()
                        .map_err(TryFromEthAbiConnectionEndError::Version)
                })
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromEthAbiConnectionEndError::State)?,
            counterparty: val
                .counterparty
                .try_into()
                .map_err(TryFromEthAbiConnectionEndError::Counterparty)?,
            delay_period: val.delay_period,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ConnectionEnd> for contracts::glue::IbcCoreConnectionV1ConnectionEndData {
    fn from(val: ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id.to_string(),
            versions: val.versions.into_iter().map(Into::into).collect(),
            state: val.state.into(),
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        }
    }
}
