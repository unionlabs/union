use unionlabs::{hash::H256, ibc::core::client::height::Height};

use crate::chain_id::ChainId;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    /// This field only ever has one of two values:
    ///
    /// - 0: client is not frozen
    /// - 1: client is frozen
    ///
    /// Both the field name and type match the ICS07 Tendermint implementation.
    ///
    /// Note that the above bounds are not enforced at the type level, which also matches the Tendermint specification.
    pub frozen_height: Height,
    pub latest_height: Height,
    /// For clients that connect to the cosmwasm implementation of union IBC, the contract address of the IBC host is required in order to verify storage proofs. For clients connecting to IBC classic, this field is not required and can be ignored during client creation and migration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "H256::is_zero")
    )]
    pub contract_address: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        hash::H256,
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
                contract_address: H256::default(),
            })
        }
    }
}
