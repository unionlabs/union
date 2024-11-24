use core::array::TryFromSliceError;

use alloy::sol_types::SolValue;
use unionlabs::{bls::BlsPublicKey, hash::H256, impl_ethabi_via_try_from_into};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
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

alloy::sol! {
    struct SolConsensusState {
        uint64 slot;
        bytes32 state_root;
        bytes32 storage_root;
        uint64 timestamp;
        bytes current_sync_committee;
        bytes next_sync_committee;
    }
}

impl From<ConsensusState> for SolConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            slot: value.slot,
            state_root: value.state_root.get().into(),
            storage_root: value.storage_root.get().into(),
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee.0.into(),
            next_sync_committee: value.next_sync_committee.0.into(),
        }
    }
}

impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

#[derive(Debug, thiserror::Error)]
pub enum TryFromEthAbiError {
    #[error("invalid current sync committee")]
    CurrentSyncCommittee(#[source] TryFromSliceError),
    #[error("invalid next sync committee")]
    NextSyncCommittee(#[source] TryFromSliceError),
}

impl TryFrom<SolConsensusState> for ConsensusState {
    type Error = TryFromEthAbiError;

    fn try_from(value: SolConsensusState) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.slot,
            state_root: H256::new(value.state_root.0),
            storage_root: H256::new(value.storage_root.0),
            timestamp: value.timestamp,
            current_sync_committee: BlsPublicKey(
                value
                    .current_sync_committee
                    .as_ref()
                    .try_into()
                    .map_err(TryFromEthAbiError::CurrentSyncCommittee)?,
            ),
            next_sync_committee: BlsPublicKey(
                value
                    .next_sync_committee
                    .as_ref()
                    .try_into()
                    .map_err(TryFromEthAbiError::NextSyncCommittee)?,
            ),
        })
    }
}
