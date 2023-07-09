use crate::{
    errors::{MissingField, UnknownEnumVariant},
    ibc::core::connection::{counterparty::Counterparty, state::State, version::Version},
    IntoProto, TypeUrl,
};

#[derive(Debug, Clone)]
pub struct ConnectionEnd {
    pub client_id: String,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty,
    pub delay_period: u64,
}

#[derive(Debug)]
pub enum TryFromConnectionEndError {
    UnknownVersion(UnknownEnumVariant<String>),
    UnknownState(UnknownEnumVariant<i32>),
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
                .map(|x| {
                    x.try_into()
                        .map_err(TryFromConnectionEndError::UnknownVersion)
                })
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromConnectionEndError::UnknownState)?,
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

impl IntoProto for ConnectionEnd {
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
pub enum TryFromEthConnectionEndError {
    UnknownVersion(UnknownEnumVariant<String>),
    UnknownState(UnknownEnumVariant<u8>),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData> for ConnectionEnd {
    type Error = TryFromEthConnectionEndError;

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
                        .map_err(TryFromEthConnectionEndError::UnknownVersion)
                })
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromEthConnectionEndError::UnknownState)?,
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        })
    }
}
