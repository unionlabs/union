use core::str::FromStr;

use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::core::commitment::merkle_prefix::MerklePrefix,
    id,
    traits::Id,
};

#[model(proto(raw(protos::ibc::core::connection::v1::Counterparty), into, from))]
#[serde(bound(
    serialize = "
        ClientId: Id,
        ConnectionId: Id,
    ",
    deserialize = "
        ClientId: Id,
        ConnectionId: Id,
    ",
))]
pub struct Counterparty<ClientId: Id, ConnectionId: Id = id::ConnectionId> {
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
    pub prefix: MerklePrefix,
}

impl<ClientId: Id, ConnectionId: Id> From<Counterparty<ClientId, ConnectionId>>
    for protos::ibc::core::connection::v1::Counterparty
{
    fn from(value: Counterparty<ClientId, ConnectionId>) -> Self {
        Self {
            client_id: value.client_id.to_string(),
            connection_id: value.connection_id.to_string(),
            prefix: Some(value.prefix.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromConnectionCounterpartyError<ClientId: Id, ConnectionId: Id> {
    MissingField(MissingField),
    ClientId(<ClientId as FromStr>::Err),
    ConnectionId(<ConnectionId as FromStr>::Err),
}

impl<ClientId: Id, ConnectionId: Id> TryFrom<protos::ibc::core::connection::v1::Counterparty>
    for Counterparty<ClientId, ConnectionId>
{
    type Error = TryFromConnectionCounterpartyError<ClientId, ConnectionId>;

    fn try_from(
        value: protos::ibc::core::connection::v1::Counterparty,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value
                .client_id
                .parse()
                .map_err(TryFromConnectionCounterpartyError::ClientId)?,
            connection_id: value
                .connection_id
                .parse()
                .map_err(TryFromConnectionCounterpartyError::ConnectionId)?,
            prefix: required!(value.prefix)?.into(),
        })
    }
}

#[cfg(feature = "ethabi")]
impl<ClientId: Id, ConnectionId: Id> From<Counterparty<ClientId, ConnectionId>>
    for contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData
{
    fn from(value: Counterparty<ClientId, ConnectionId>) -> Self {
        Self {
            client_id: value.client_id.to_string(),
            connection_id: value.connection_id.to_string(),
            prefix: value.prefix.into(),
        }
    }
}

#[derive(Debug)]
#[cfg(feature = "ethabi")]
pub enum TryFromEthAbiConnectionCounterpartyError<ClientId: Id, ConnectionId: Id> {
    ClientId(<ClientId as FromStr>::Err),
    ConnectionId(<ConnectionId as FromStr>::Err),
}

#[cfg(feature = "ethabi")]
impl<ClientId: Id, ConnectionId: Id>
    TryFrom<contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData>
    for Counterparty<ClientId, ConnectionId>
{
    type Error = TryFromEthAbiConnectionCounterpartyError<ClientId, ConnectionId>;

    fn try_from(
        value: contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value
                .client_id
                .parse()
                .map_err(TryFromEthAbiConnectionCounterpartyError::ClientId)?,
            connection_id: value
                .connection_id
                .parse()
                .map_err(TryFromEthAbiConnectionCounterpartyError::ConnectionId)?,
            prefix: value.prefix.into(),
        })
    }
}
