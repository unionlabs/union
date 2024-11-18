use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into};

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::arbitrum::v1::ConsensusState);

    impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::arbitrum::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                ibc_storage_root: value
                    .ibc_storage_root
                    .try_into()
                    .map_err(Error::IbcStorageRoot)?,
                timestamp: value.timestamp,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid ibc storage root")]
        IbcStorageRoot(#[source] InvalidLength),
    }

    impl From<ConsensusState> for protos::union::ibc::lightclients::arbitrum::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                ibc_storage_root: value.ibc_storage_root.into(),
                timestamp: value.timestamp,
            }
        }
    }
}
