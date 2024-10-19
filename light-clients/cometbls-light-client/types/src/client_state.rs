use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::Height;

use crate::chain_id::ChainId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub frozen_height: Height,
    pub latest_height: Height,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into, required,
    };

    use crate::{client_state::ClientState, ChainId};

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::cometbls::v1::ClientState);

    impl From<ClientState> for protos::union::ibc::lightclients::cometbls::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                chain_id: value.chain_id.to_string(),
                trusting_period: value.trusting_period,
                max_clock_drift: value.max_clock_drift,
                frozen_height: Some(value.frozen_height.into()),
                latest_height: Some(value.latest_height.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid chain_id")]
        ChainId(#[from] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: ChainId::from_string(value.chain_id)?,
                trusting_period: value.trusting_period,
                max_clock_drift: value.max_clock_drift,
                frozen_height: required!(value.frozen_height)?.into(),
                latest_height: required!(value.latest_height)?.into(),
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::string::FromUtf8Error;

    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Decode, Encode, EthAbi},
        ibc::core::client::height::Height,
        TryFromEthAbiBytesErrorAlloy,
    };

    use crate::{ChainId, ClientState};

    alloy::sol! {
        struct SolClientState {
            bytes31 chainId;
            uint64 trustingPeriod;
            uint64 maxClockDrift;
            uint64 frozenHeight;
            uint64 latestHeight;
        }
    }

    impl Encode<EthAbi> for ClientState {
        fn encode(self) -> Vec<u8> {
            SolClientState {
                chainId: self.chain_id.into_fixed_bytes(),
                trustingPeriod: self.trusting_period,
                maxClockDrift: self.max_clock_drift,
                frozenHeight: self.frozen_height.revision_height,
                latestHeight: self.latest_height.revision_height,
            }
            .abi_encode()
        }
    }

    impl Decode<EthAbi> for ClientState {
        type Error = TryFromEthAbiBytesErrorAlloy<Error>;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let client_state = SolClientState::abi_decode(bytes, true)?;

            Ok(Self {
                chain_id: ChainId::try_from_fixed_bytes(client_state.chainId)
                    .map_err(|err| TryFromEthAbiBytesErrorAlloy::Convert(Error::ChainId(err)))?,
                trusting_period: client_state.trustingPeriod,
                max_clock_drift: client_state.maxClockDrift,
                frozen_height: Height {
                    revision_number: 0,
                    revision_height: client_state.frozenHeight,
                },
                latest_height: Height {
                    revision_number: 0,
                    revision_height: client_state.latestHeight,
                },
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid chain_id")]
        ChainId(#[from] FromUtf8Error),
    }
}
