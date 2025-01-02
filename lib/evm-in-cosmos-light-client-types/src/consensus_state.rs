use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub evm_state_root: H256,
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            bytes32 evm_state_root;
            bytes32 ibc_storage_root;
            uint64 timestamp;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                evm_storage_root: value.evm_storage_root.get().into(),
                ibc_storage_root: value.root.hash.get().into(),
                timestamp: value.timestamp.as_unix_nanos(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                evm_storage_root: H256::new(value.evm_storage_root.0).into(),
                ibc_storage_root: H256::new(value.ibc_storage_root.0).into(),
                timestamp: Timestamp::try_from_unix_nanos(value.timestamp.into())
                    .expect("impossible"),
            }
        }
    }
}

// impl From<ConsensusState> for protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState {
//     fn from(value: ConsensusState) -> Self {
//         Self {
//             evm_state_root: value.evm_state_root.into(),
//             ibc_storage_root: value.ibc_storage_root.into(),
//             timestamp: value.timestamp,
//         }
//     }
// }

// #[derive(Debug, PartialEq, Clone, thiserror::Error)]
// pub enum TryFromConsensusStateError {
//     #[error("invalid evm state root")]
//     EvmStateRoot(#[source] InvalidLength),
//     #[error("invalid ibc storage root")]
//     IbcStorageRoot(#[source] InvalidLength),
// }

// impl TryFrom<protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState> for ConsensusState {
//     type Error = TryFromConsensusStateError;
//     fn try_from(
//         value: protos::union::ibc::lightclients::evmincosmos::v1::ConsensusState,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self {
//             evm_state_root: value
//                 .evm_state_root
//                 .try_into()
//                 .map_err(TryFromConsensusStateError::EvmStateRoot)?,
//             ibc_storage_root: value
//                 .ibc_storage_root
//                 .try_into()
//                 .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
//             timestamp: value.timestamp,
//         })
//     }
// }
