use unionlabs::{
    bls::BlsSignature,
    hash::{hash_v2::Hash, H256},
};
#[cfg(feature = "ssz")]
use {
    crate::{
        AttestationSsz, AttesterSlashingSsz, DepositSsz, ExecutionPayloadSsz, SyncAggregateSsz,
        BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
        MAX_ATTESTER_SLASHINGS, MAX_BLOB_COMMITMENTS_PER_BLOCK, MAX_BLS_TO_EXECUTION_CHANGES,
        MAX_BYTES_PER_TRANSACTION, MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS,
        MAX_TRANSACTIONS_PER_PAYLOAD, MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS,
        MAX_WITHDRAWALS_PER_PAYLOAD, SYNC_COMMITTEE_SIZE,
    },
    ssz::{types::List, Ssz},
};

use crate::{
    sync_aggregate::SyncAggregate, Attestation, AttesterSlashing, Deposit, Eth1Data,
    ExecutionPayload, ProposerSlashing, SignedBlsToExecutionChange, SignedVoluntaryExit,
};

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct BeaconBlockBodySsz<
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
    pub randao_reveal: BlsSignature,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: List<ProposerSlashing, C::MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings: List<AttesterSlashingSsz<C>, C::MAX_ATTESTER_SLASHINGS>,
    pub attestations: List<AttestationSsz<C>, C::MAX_ATTESTATIONS>,
    pub deposits: List<DepositSsz<C>, C::MAX_DEPOSITS>,
    pub voluntary_exits: List<SignedVoluntaryExit, C::MAX_VOLUNTARY_EXITS>,
    pub sync_aggregate: SyncAggregateSsz<C>,
    pub execution_payload: ExecutionPayloadSsz<C>,
    pub bls_to_execution_changes: List<SignedBlsToExecutionChange, C::MAX_BLS_TO_EXECUTION_CHANGES>,
    pub blob_kzg_commitments: List<Hash<48>, C::MAX_BLOB_COMMITMENTS_PER_BLOCK>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlockBody {
    pub randao_reveal: BlsSignature,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: Vec<ProposerSlashing>,
    pub attester_slashings: Vec<AttesterSlashing>,
    pub attestations: Vec<Attestation>,
    pub deposits: Vec<Deposit>,
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
    pub sync_aggregate: SyncAggregate,
    pub execution_payload: ExecutionPayload,
    pub bls_to_execution_changes: Vec<SignedBlsToExecutionChange>,
    pub blob_kzg_commitments: Vec<Hash<48>>,
}
