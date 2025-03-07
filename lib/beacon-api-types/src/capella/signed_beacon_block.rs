use unionlabs::primitives::H768;

use crate::capella::BeaconBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SignedBeaconBlockSsz<
    C: crate::chain_spec::MAX_PROPOSER_SLASHINGS
        + crate::chain_spec::MAX_VALIDATORS_PER_COMMITTEE
        + crate::chain_spec::MAX_ATTESTER_SLASHINGS
        + crate::chain_spec::MAX_ATTESTATIONS
        + crate::chain_spec::DEPOSIT_CONTRACT_TREE_DEPTH
        + crate::chain_spec::MAX_DEPOSITS
        + crate::chain_spec::MAX_VOLUNTARY_EXITS
        + crate::chain_spec::BYTES_PER_LOGS_BLOOM
        + crate::chain_spec::MAX_EXTRA_DATA_BYTES
        + crate::chain_spec::MAX_BYTES_PER_TRANSACTION
        + crate::chain_spec::MAX_TRANSACTIONS_PER_PAYLOAD
        + crate::chain_spec::MAX_WITHDRAWALS_PER_PAYLOAD
        + crate::chain_spec::MAX_BLS_TO_EXECUTION_CHANGES
        + crate::chain_spec::MAX_BLOB_COMMITMENTS_PER_BLOCK
        + crate::chain_spec::SYNC_COMMITTEE_SIZE,
> {
    pub message: crate::capella::BeaconBlockSsz<C>,
    pub signature: H768,
}
