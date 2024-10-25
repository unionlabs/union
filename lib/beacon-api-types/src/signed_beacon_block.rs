use unionlabs::bls::BlsSignature;

use crate::{BeaconBlock, BeaconBlockSsz};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: BlsSignature,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SignedBeaconBlockSsz<
    C: crate::MAX_PROPOSER_SLASHINGS
        + crate::MAX_VALIDATORS_PER_COMMITTEE
        + crate::MAX_ATTESTER_SLASHINGS
        + crate::MAX_ATTESTATIONS
        + crate::DEPOSIT_CONTRACT_TREE_DEPTH
        + crate::MAX_DEPOSITS
        + crate::MAX_VOLUNTARY_EXITS
        + crate::BYTES_PER_LOGS_BLOOM
        + crate::MAX_EXTRA_DATA_BYTES
        + crate::MAX_BYTES_PER_TRANSACTION
        + crate::MAX_TRANSACTIONS_PER_PAYLOAD
        + crate::MAX_WITHDRAWALS_PER_PAYLOAD
        + crate::MAX_BLS_TO_EXECUTION_CHANGES
        + crate::MAX_BLOB_COMMITMENTS_PER_BLOCK
        + crate::SYNC_COMMITTEE_SIZE,
> {
    pub message: BeaconBlockSsz<C>,
    pub signature: BlsSignature,
}
