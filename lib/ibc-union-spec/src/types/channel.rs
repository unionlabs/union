use unionlabs::primitives::Bytes;

use crate::types::{ChannelId, ConnectionId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Channel {
    pub state: Option<ChannelState>,
    pub connection_id: ConnectionId,
    pub counterparty_channel_id: ChannelId,
    pub counterparty_port_id: Bytes,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub enum ChannelState {
    Init,
    TryOpen,
    Open,
    Closed,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy_sol_types::SolValue;

    use super::*;

    unionlabs::impl_ethabi_via_try_from_into!(Channel => ibc_solidity::Channel);

    impl From<Channel> for ibc_solidity::Channel {
        fn from(value: Channel) -> Self {
            Self {
                state: match value.state {
                    None => ibc_solidity::ChannelState::Unspecified,
                    Some(ChannelState::Init) => ibc_solidity::ChannelState::Init,
                    Some(ChannelState::TryOpen) => ibc_solidity::ChannelState::TryOpen,
                    Some(ChannelState::Open) => ibc_solidity::ChannelState::Open,
                    Some(ChannelState::Closed) => ibc_solidity::ChannelState::Closed,
                },
                connection_id: value.connection_id,
                counterparty_channel_id: value.counterparty_channel_id,
                counterparty_port_id: value.counterparty_port_id.into(),
                version: value.version,
            }
        }
    }

    impl TryFrom<ibc_solidity::Channel> for Channel {
        type Error = Error;

        fn try_from(value: ibc_solidity::Channel) -> Result<Self, Self::Error> {
            Ok(Self {
                state: match value.state {
                    ibc_solidity::ChannelState::Unspecified => None,
                    ibc_solidity::ChannelState::Init => Some(ChannelState::Init),
                    ibc_solidity::ChannelState::TryOpen => Some(ChannelState::TryOpen),
                    ibc_solidity::ChannelState::Open => Some(ChannelState::Open),
                    ibc_solidity::ChannelState::Closed => Some(ChannelState::Closed),
                    // ???
                    ibc_solidity::ChannelState::__Invalid => {
                        return Err(Error::InvalidChannelState)
                    }
                },
                connection_id: value.connection_id,
                counterparty_channel_id: value.counterparty_channel_id,
                counterparty_port_id: value.counterparty_port_id.into(),
                version: value.version,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid channel state")]
        InvalidChannelState,
    }
}
