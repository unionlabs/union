use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField, UnknownEnumVariant},
    ibc::core::connection::{counterparty::Counterparty, state::State, version::Version},
    id,
    traits::Id,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionEnd<
    ClientId,
    CounterpartyClientId,
    CounterpartyConnectionId = id::ConnectionId,
> {
    pub client_id: ClientId,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty<CounterpartyClientId, CounterpartyConnectionId>,
    pub delay_period: u64,
}

#[derive(Debug)]
pub enum TryFromConnectionEndError<
    ClientId: Id,
    CounterpartyClientId: Id,
    CounterpartyConnectionId: Id,
> {
    ClientId(<ClientId as FromStr>::Err),
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<i32>),
    Counterparty(TryFromProtoErrorOf<Counterparty<CounterpartyClientId, CounterpartyConnectionId>>),
    MissingField(MissingField),
}

impl<ClientId: Id, CounterpartyClientId: Id, CounterpartyConnectionId: Id>
    TryFrom<protos::ibc::core::connection::v1::ConnectionEnd>
    for ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>
{
    type Error =
        TryFromConnectionEndError<ClientId, CounterpartyClientId, CounterpartyConnectionId>;

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

impl<ClientId: Id, CounterpartyClientId: Id, CounterpartyConnectionId: Id> Proto
    for ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>
{
    type Proto = protos::ibc::core::connection::v1::ConnectionEnd;
}

impl TypeUrl for protos::ibc::core::connection::v1::ConnectionEnd {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.ConnectionEnd";
}

impl<ClientId: Id, CounterpartyClientId: Id> From<ConnectionEnd<ClientId, CounterpartyClientId>>
    for protos::ibc::core::connection::v1::ConnectionEnd
{
    fn from(val: ConnectionEnd<ClientId, CounterpartyClientId>) -> Self {
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
pub enum TryFromEthAbiConnectionEndError<
    ClientId: Id,
    CounterpartyClientId: Id,
    CounterpartyConnectionId: Id,
> {
    ClientId(<ClientId as FromStr>::Err),
    Version(UnknownEnumVariant<String>),
    State(UnknownEnumVariant<u8>),
    Counterparty(
        crate::TryFromEthAbiErrorOf<Counterparty<CounterpartyClientId, CounterpartyConnectionId>>,
    ),
}

#[cfg(feature = "ethabi")]
impl<ClientId: Id, CounterpartyClientId: Id, CounterpartyConnectionId: Id>
    TryFrom<contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData>
    for ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>
{
    type Error =
        TryFromEthAbiConnectionEndError<ClientId, CounterpartyClientId, CounterpartyConnectionId>;

    fn try_from(
        val: contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData,
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
impl<ClientId: Id, CounterpartyClientId: Id, CounterpartyConnectionId: Id>
    From<ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>>
    for contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData
{
    fn from(val: ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>) -> Self {
        Self {
            client_id: val.client_id.to_string(),
            versions: val.versions.into_iter().map(Into::into).collect(),
            state: val.state.into(),
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        }
    }
}

#[cfg(feature = "ethabi")]
impl<ClientId: Id, CounterpartyClientId: Id, CounterpartyConnectionId: Id> crate::EthAbi
    for ConnectionEnd<ClientId, CounterpartyClientId, CounterpartyConnectionId>
{
    type EthAbi = contracts::ibc_handler::IbcCoreConnectionV1ConnectionEndData;
}
