use unionlabs::{bls::BlsPublicKey, hash::H256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    // REVIEW: Remove this field as this height is what is used to query the consensus state?
    pub slot: u64,
    /// The state root for this chain, used for L2s to verify against this contract.
    pub state_root: H256,
    pub storage_root: H256,
    /// Timestamp of the block, *normalized to nanoseconds* in order to be compatible with ibc-go.
    pub timestamp: u64,
    /// aggregate public key of current sync committee
    pub current_sync_committee: BlsPublicKey,
    /// aggregate public key of next sync committee
    pub next_sync_committee: BlsPublicKey,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into};

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::ethereum::v1::ConsensusState);

    impl From<ConsensusState> for protos::union::ibc::lightclients::ethereum::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                slot: value.slot,
                state_root: value.state_root.into(),
                storage_root: value.storage_root.into(),
                timestamp: value.timestamp,
                current_sync_committee: value.current_sync_committee.into(),
                next_sync_committee: value.next_sync_committee.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid current sync committee")]
        CurrentSyncCommittee(#[source] InvalidLength),
        #[error("invalid next sync committee")]
        NextSyncCommittee(#[source] InvalidLength),
        #[error("invalid storage root")]
        StorageRoot(#[source] InvalidLength),
        #[error("invalid state root")]
        StateRoot(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                slot: value.slot,
                state_root: value.state_root.try_into().map_err(Error::StorageRoot)?,
                storage_root: value.storage_root.try_into().map_err(Error::StorageRoot)?,
                timestamp: value.timestamp,
                current_sync_committee: value
                    .current_sync_committee
                    .try_into()
                    .map_err(Error::CurrentSyncCommittee)?,
                next_sync_committee: value
                    .next_sync_committee
                    .try_into()
                    .map_err(Error::NextSyncCommittee)?,
            })
        }
    }
}
