use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

#[cfg(feature = "proto")]
pub mod porto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into};

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::linea::v1::ConsensusState);

    impl From<ConsensusState> for protos::union::ibc::lightclients::linea::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                ibc_storage_root: value.ibc_storage_root.into(),
                timestamp: value.timestamp,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromConsensusStateError {
        #[error("invalid ibc storage root")]
        IbcStorageRoot(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                ibc_storage_root: value
                    .ibc_storage_root
                    .try_into()
                    .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
                timestamp: value.timestamp,
            })
        }
    }
}
