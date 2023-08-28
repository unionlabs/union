use serde::Serialize;

use crate::{
    errors::{MissingField, UnknownEnumVariant},
    ibc::core::connection::{counterparty::Counterparty, state::State, version::Version},
    Proto, TypeUrl,
};

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionEnd {
    pub client_id: String,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty,
    pub delay_period: u64,
}

#[derive(Debug)]
pub enum TryFromConnectionEndError {
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<i32>),
    MissingField(MissingField),
}

impl TryFrom<protos::ibc::core::connection::v1::ConnectionEnd> for ConnectionEnd {
    type Error = TryFromConnectionEndError;

    fn try_from(
        val: protos::ibc::core::connection::v1::ConnectionEnd,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val.client_id,
            versions: val
                .versions
                .into_iter()
                .map(|x| x.try_into().map_err(TryFromConnectionEndError::Version))
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromConnectionEndError::State)?,
            counterparty: val
                .counterparty
                .ok_or(TryFromConnectionEndError::MissingField(MissingField(
                    "counterparty",
                )))?
                .try_into()
                .map_err(TryFromConnectionEndError::MissingField)?,
            delay_period: val.delay_period,
        })
    }
}

impl Proto for ConnectionEnd {
    type Proto = protos::ibc::core::connection::v1::ConnectionEnd;
}

impl TypeUrl for protos::ibc::core::connection::v1::ConnectionEnd {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.ConnectionEnd";
}

impl From<ConnectionEnd> for protos::ibc::core::connection::v1::ConnectionEnd {
    fn from(val: ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id,
            versions: val.versions.into_iter().map(Into::into).collect(),
            state: val.state as i32,
            counterparty: Some(val.counterparty.into()),
            delay_period: val.delay_period,
        }
    }
}

#[derive(Debug)]
pub enum TryFromEthAbiConnectionEndError {
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<u8>),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData> for ConnectionEnd {
    type Error = TryFromEthAbiConnectionEndError;

    fn try_from(
        val: contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val.client_id,
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
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ConnectionEnd> for contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData {
    fn from(val: ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id,
            versions: val.versions.into_iter().map(Into::into).collect(),
            state: val.state as u8,
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for ConnectionEnd {
    type EthAbi = contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData;
}
