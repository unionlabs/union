use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub state_root: H256,
    pub timestamp: u64,
    pub ibc_storage_root: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into};

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::scroll::v1::ConsensusState);

    impl From<ConsensusState> for protos::union::ibc::lightclients::scroll::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                state_root: value.state_root.into(),
                timestamp: value.timestamp,
                ibc_storage_root: value.ibc_storage_root.into(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("invalid state root")]
        StateRoot(#[source] InvalidLength),
        #[error("invalid ibc storage root")]
        IbcStorageRoot(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::scroll::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                state_root: value.state_root.try_into().map_err(Error::IbcStorageRoot)?,
                timestamp: value.timestamp,
                ibc_storage_root: value
                    .ibc_storage_root
                    .try_into()
                    .map_err(Error::IbcStorageRoot)?,
            })
        }
    }
}
