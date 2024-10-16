use macros::model;

use crate::{
    ibc::core::connection::{counterparty::Counterparty, state::State, version::Version},
    id::ClientId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::ConnectionEnd), into))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
#[cfg_attr(feature = "valuable", derive(::valuable::Valuable))]
pub struct ConnectionEnd {
    pub client_id: ClientId,
    pub versions: Vec<Version>,
    pub state: State,
    pub counterparty: Counterparty,
    pub delay_period: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField, UnknownEnumVariant},
        ibc::core::connection::{
            connection_end::ConnectionEnd, counterparty::proto::TryFromConnectionCounterpartyError,
        },
        id::{ClientId, ParsePrefixedIdError},
    };

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromConnectionEndError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid client_id")]
        ClientId(#[from] ParsePrefixedIdError),
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
            value: protos::ibc::core::connection::v1::ConnectionEnd,
        ) -> Result<Self, Self::Error> {
            let (_, client_id) = ClientId::parse_prefixed(&value.client_id)
                .map_err(TryFromConnectionEndError::ClientId)?;

            Ok(Self {
                client_id,
                versions: value
                    .versions
                    .into_iter()
                    .map(|x| x.try_into().map_err(TryFromConnectionEndError::Version))
                    .collect::<Result<_, _>>()?,
                state: value
                    .state
                    .try_into()
                    .map_err(TryFromConnectionEndError::State)?,
                counterparty: required!(value.counterparty)?
                    .try_into()
                    .map_err(TryFromConnectionEndError::Counterparty)?,
                delay_period: value.delay_period,
            })
        }
    }

    // impl From<ConnectionEnd> for protos::ibc::core::connection::v1::ConnectionEnd {
    //     fn from(value: ConnectionEnd) -> Self {
    //         Self {
    //             client_id: value.client_id.to_string_prefixed(&value.client_type),
    //             versions: value.versions.into_iter().map(Into::into).collect(),
    //             state: value.state as i32,
    //             counterparty: Some(value.counterparty.into()),
    //             delay_period: value.delay_period,
    //         }
    //     }
    // }
}
