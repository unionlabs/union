use core::str::FromStr;

use macros::model;

use crate::{
    errors::{MissingField, UnknownEnumVariant, required},
    ibc::core::connection::{
        counterparty::{Counterparty, TryFromConnectionCounterpartyError},
        state::State,
        version::Version,
    },
    id::ClientId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::ConnectionEnd), into, from))]
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
