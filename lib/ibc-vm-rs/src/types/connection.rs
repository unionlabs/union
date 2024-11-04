use serde::{Deserialize, Serialize};
use unionlabs::id::ConnectionId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Unspecified,
    Init,
    TryOpen,
    Open,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEnd {
    pub state: ConnectionState,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: ConnectionId,
}

impl TryFrom<ibc_solidity::ibc::Connection> for ConnectionEnd {
    type Error = TryFromConnectionEndError;

    fn try_from(value: ibc_solidity::ibc::Connection) -> Result<Self, Self::Error> {
        Ok(Self {
            state: match value.state {
                ibc_solidity::ibc::ConnectionState::Unspecified => ConnectionState::Unspecified,
                ibc_solidity::ibc::ConnectionState::Init => ConnectionState::Init,
                ibc_solidity::ibc::ConnectionState::TryOpen => ConnectionState::TryOpen,
                ibc_solidity::ibc::ConnectionState::Open => ConnectionState::Open,
                _ => return Err(TryFromConnectionEndError::InvalidState),
            },
            client_id: value.clientId,
            counterparty_client_id: value.counterpartyClientId,
            counterparty_connection_id: ConnectionId::new(value.counterpartyConnectionId),
        })
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromConnectionEndError {
    #[error("connection state is invalid")]
    InvalidState,
}

impl From<ConnectionEnd> for ibc_solidity::ibc::Connection {
    fn from(value: ConnectionEnd) -> Self {
        Self {
            state: match value.state {
                ConnectionState::Unspecified => ibc_solidity::ibc::ConnectionState::Unspecified,
                ConnectionState::Init => ibc_solidity::ibc::ConnectionState::Init,
                ConnectionState::TryOpen => ibc_solidity::ibc::ConnectionState::TryOpen,
                ConnectionState::Open => ibc_solidity::ibc::ConnectionState::Open,
            },
            clientId: value.client_id,
            counterpartyClientId: value.counterparty_client_id,
            counterpartyConnectionId: value.counterparty_connection_id.id(),
        }
    }
}
