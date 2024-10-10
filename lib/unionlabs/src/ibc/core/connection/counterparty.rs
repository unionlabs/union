use core::str::FromStr;

use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::core::commitment::merkle_prefix::MerklePrefix,
    id::{ClientId, ConnectionId},
};

#[model(proto(raw(protos::ibc::core::connection::v1::Counterparty), into, from))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct Counterparty {
    pub client_id: ClientId,
    // this is really `Either<ConnectionId, EmptyString>`
    pub connection_id: Option<ConnectionId>,
    pub prefix: MerklePrefix,
}

impl From<Counterparty> for protos::ibc::core::connection::v1::Counterparty {
    fn from(value: Counterparty) -> Self {
        Self {
            client_id: value.client_id.to_string(),
            connection_id: value
                .connection_id
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            prefix: Some(value.prefix.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromConnectionCounterpartyError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid client_id")]
    ClientId(#[source] <ClientId as FromStr>::Err),
    #[error("invalid connection_id")]
    ConnectionId(#[source] <ConnectionId as FromStr>::Err),
}

impl TryFrom<protos::ibc::core::connection::v1::Counterparty> for Counterparty {
    type Error = TryFromConnectionCounterpartyError;

    fn try_from(
        value: protos::ibc::core::connection::v1::Counterparty,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value
                .client_id
                .parse()
                .map_err(TryFromConnectionCounterpartyError::ClientId)?,
            connection_id: if value.connection_id.is_empty() {
                None
            } else {
                Some(
                    value
                        .connection_id
                        .parse()
                        .map_err(TryFromConnectionCounterpartyError::ConnectionId)?,
                )
            },
            prefix: required!(value.prefix)?.into(),
        })
    }
}
