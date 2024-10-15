use macros::model;

use crate::{
    ibc::core::commitment::merkle_prefix::MerklePrefix,
    id::{ClientId, ConnectionId},
};

#[model(proto(raw(protos::ibc::core::connection::v1::Counterparty), into, from))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
#[cfg_attr(feature = "valuable", derive(::valuable::Valuable))]
pub struct Counterparty {
    pub client_id: ClientId,
    pub client_type: String,
    // this is really `Either<ConnectionId, EmptyString>`
    pub connection_id: Option<ConnectionId>,
    pub prefix: MerklePrefix,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField},
        ibc::core::connection::counterparty::Counterparty,
        id::{ClientId, ConnectionId, ParsePrefixedIdError},
    };

    impl From<Counterparty> for protos::ibc::core::connection::v1::Counterparty {
        fn from(value: Counterparty) -> Self {
            Self {
                client_id: value.client_id.to_string_prefixed(&value.client_type),
                connection_id: value
                    .connection_id
                    .as_ref()
                    .map_or_else(String::new, ConnectionId::to_string_prefixed),
                prefix: Some(value.prefix.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromConnectionCounterpartyError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid client_id")]
        ClientId(#[source] ParsePrefixedIdError),
        #[error("invalid connection_id")]
        ConnectionId(#[source] ParsePrefixedIdError),
    }

    impl TryFrom<protos::ibc::core::connection::v1::Counterparty> for Counterparty {
        type Error = TryFromConnectionCounterpartyError;

        fn try_from(
            value: protos::ibc::core::connection::v1::Counterparty,
        ) -> Result<Self, Self::Error> {
            let (client_type, client_id) = ClientId::parse_prefixed(&value.client_id)
                .map_err(TryFromConnectionCounterpartyError::ClientId)?;

            Ok(Self {
                client_id,
                client_type: client_type.to_owned(),
                connection_id: if value.connection_id.is_empty() {
                    None
                } else {
                    ConnectionId::parse_prefixed(&value.connection_id)
                        .map(Some)
                        .map_err(TryFromConnectionCounterpartyError::ConnectionId)?
                },
                prefix: required!(value.prefix)?.into(),
            })
        }
    }
}
