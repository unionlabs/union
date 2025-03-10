use unionlabs::primitives::{H256, H768};
#[cfg(feature = "ssz")]
use {
    crate::{
        altair::SyncAggregateSsz,
        bellatrix::ExecutionPayloadSsz,
        chain_spec::{
            BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
            MAX_ATTESTER_SLASHINGS, MAX_BLOB_COMMITMENTS_PER_BLOCK, MAX_BLS_TO_EXECUTION_CHANGES,
            MAX_BYTES_PER_TRANSACTION, MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS,
            MAX_TRANSACTIONS_PER_PAYLOAD, MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS,
            MAX_WITHDRAWALS_PER_PAYLOAD, SYNC_COMMITTEE_SIZE,
        },
        phase0::{AttestationSsz, AttesterSlashingSsz, DepositSsz},
    },
    ssz::{types::List, Ssz},
};

use crate::{
    altair::SyncAggregate,
    bellatrix::execution_payload::ExecutionPayload,
    capella::SignedBlsToExecutionChange,
    phase0::{
        Attestation, AttesterSlashing, Deposit, Eth1Data, ProposerSlashing, SignedVoluntaryExit,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlockBody {
    pub randao_reveal: H768,
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
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, Ssz)]
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
    pub randao_reveal: H768,
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
}
