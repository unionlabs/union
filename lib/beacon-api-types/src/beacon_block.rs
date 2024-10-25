#[cfg(feature = "ssz")]
use unionlabs::ethereum::config::{
    BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS, MAX_ATTESTER_SLASHINGS,
    MAX_BLOB_COMMITMENTS_PER_BLOCK, MAX_BLS_TO_EXECUTION_CHANGES, MAX_BYTES_PER_TRANSACTION,
    MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS, MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS, MAX_WITHDRAWALS_PER_PAYLOAD,
    SYNC_COMMITTEE_SIZE,
};

use crate::BeaconBlockBody;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlock {
    #[serde(with = "::serde_utils::string")]
    pub slot: u64,
    #[serde(with = "::serde_utils::string")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct BeaconBlockSsz<
    C: MAX_PROPOSER_SLASHINGS
        + MAX_VALIDATORS_PER_COMMITTEE
        + MAX_ATTESTER_SLASHINGS
        + MAX_ATTESTATIONS
        + DEPOSIT_CONTRACT_TREE_DEPTH
        + MAX_DEPOSITS
        + MAX_VOLUNTARY_EXITS
        + BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD
        + MAX_BLS_TO_EXECUTION_CHANGES
        + MAX_BLOB_COMMITMENTS_PER_BLOCK
        + SYNC_COMMITTEE_SIZE,
> {
    #[serde(with = "::serde_utils::string")]
    pub slot: u64,
    #[serde(with = "::serde_utils::string")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
}

#[cfg(feature = "ssz")]
impl<C> BeaconBlock<C>
where
    C: MAX_PROPOSER_SLASHINGS
        + MAX_VALIDATORS_PER_COMMITTEE
        + MAX_ATTESTER_SLASHINGS
        + MAX_ATTESTATIONS
        + DEPOSIT_CONTRACT_TREE_DEPTH
        + MAX_DEPOSITS
        + MAX_VOLUNTARY_EXITS
        + BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD
        + MAX_BLS_TO_EXECUTION_CHANGES
        + MAX_BLOB_COMMITMENTS_PER_BLOCK
        + SYNC_COMMITTEE_SIZE,
{
    #[must_use]
    pub fn to_header(self) -> BeaconBlockHeader {
        BeaconBlockHeader {
            slot: self.slot,
            proposer_index: self.proposer_index,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: self.body.tree_hash_root().into(),
        }
    }
}
