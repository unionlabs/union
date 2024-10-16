use macros::model;

use crate::hash::H256;

#[model(proto(
    raw(protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    pub evm_state_root: H256,
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::InvalidLength, ibc::lightclients::evm_in_cosmos::consensus_state::ConsensusState,
    };

    impl From<ConsensusState> for protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                evm_state_root: value.evm_state_root.into(),
                ibc_storage_root: value.ibc_storage_root.into(),
                timestamp: value.timestamp,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromConsensusStateError {
        #[error("invalid evm state root")]
        EvmStateRoot(#[source] InvalidLength),
        #[error("invalid ibc storage root")]
        IbcStorageRoot(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;
        fn try_from(
            value: protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                evm_state_root: value
                    .evm_state_root
                    .try_into()
                    .map_err(TryFromConsensusStateError::EvmStateRoot)?,
                ibc_storage_root: value
                    .ibc_storage_root
                    .try_into()
                    .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
                timestamp: value.timestamp,
            })
        }
    }
}
