use crate::types::{ClientId, ConnectionId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Connection {
    pub state: Option<ConnectionState>,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub enum ConnectionState {
    Init,
    TryOpen,
    Open,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy_sol_types::SolValue;

    use super::*;

    unionlabs::impl_ethabi_via_try_from_into!(Connection => ibc_solidity::Connection);

    impl From<Connection> for ibc_solidity::Connection {
        fn from(value: Connection) -> Self {
            Self {
                state: match value.state {
                    None => ibc_solidity::ConnectionState::Unspecified,
                    Some(ConnectionState::Init) => ibc_solidity::ConnectionState::Init,
                    Some(ConnectionState::TryOpen) => ibc_solidity::ConnectionState::TryOpen,
                    Some(ConnectionState::Open) => ibc_solidity::ConnectionState::Open,
                },
                client_id: value.client_id,
                counterparty_client_id: value.counterparty_client_id,
                counterparty_connection_id: value.counterparty_connection_id,
            }
        }
    }

    impl TryFrom<ibc_solidity::Connection> for Connection {
        type Error = Error;

        fn try_from(value: ibc_solidity::Connection) -> Result<Self, Self::Error> {
            Ok(Self {
                state: match value.state {
                    ibc_solidity::ConnectionState::Unspecified => None,
                    ibc_solidity::ConnectionState::Init => Some(ConnectionState::Init),
                    ibc_solidity::ConnectionState::TryOpen => Some(ConnectionState::TryOpen),
                    ibc_solidity::ConnectionState::Open => Some(ConnectionState::Open),
                    // ???
                    ibc_solidity::ConnectionState::__Invalid => {
                        return Err(Error::InvalidConnectionState)
                    }
                },
                client_id: value.client_id,
                counterparty_client_id: value.counterparty_client_id,
                counterparty_connection_id: value.counterparty_connection_id,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid connection state")]
        InvalidConnectionState,
    }
}
