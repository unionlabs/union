use macros::model;
use ssz::Ssz;

use crate::{
    ethereum::{
        beacon::beacon_block_body::{BeaconBlockBody, UnboundedBeaconBlockBody},
        config::{
            BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
            MAX_ATTESTER_SLASHINGS, MAX_BLOB_COMMITMENTS_PER_BLOCK, MAX_BLS_TO_EXECUTION_CHANGES,
            MAX_BYTES_PER_TRANSACTION, MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS,
            MAX_TRANSACTIONS_PER_PAYLOAD, MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS,
            MAX_WITHDRAWALS_PER_PAYLOAD, SYNC_COMMITTEE_SIZE,
        },
    },
    hash::H256,
    ibc::lightclients::ethereum::beacon_block_header::BeaconBlockHeader,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
#[model]
#[cfg_attr(feature = "ssz", derive(Ssz))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct BeaconBlock<
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
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub slot: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
}

impl<
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
    > BeaconBlock<C>
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

#[model]
pub struct UnboundedBeaconBlock {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub slot: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: UnboundedBeaconBlockBody,
}
