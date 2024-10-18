use macros::model;
#[cfg(feature = "ssz")]
use {
    crate::{
        ethereum::{
            beacon::{
                attestation::Attestation, attester_slashing::AttesterSlashing, deposit::Deposit,
                execution_payload::ExecutionPayload,
            },
            config::{
                BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
                MAX_ATTESTER_SLASHINGS, MAX_BLOB_COMMITMENTS_PER_BLOCK,
                MAX_BLS_TO_EXECUTION_CHANGES, MAX_BYTES_PER_TRANSACTION, MAX_DEPOSITS,
                MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS, MAX_TRANSACTIONS_PER_PAYLOAD,
                MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS, MAX_WITHDRAWALS_PER_PAYLOAD,
                SYNC_COMMITTEE_SIZE,
            },
        },
        ibc::lightclients::ethereum::sync_aggregate::SyncAggregate,
    },
    ssz::{types::List, Ssz},
};

use crate::{
    ethereum::beacon::{
        attestation::UnboundedAttestation, attester_slashing::UnboundedAttesterSlashing,
        deposit::UnboundedDeposit, eth1_data::Eth1Data,
        execution_payload::UnboundedExecutionPayload, kzg_commitment::KZGCommitment,
        proposer_slashing::ProposerSlashing,
        signed_bls_to_execution_change::SignedBlsToExecutionChange,
        signed_voluntary_exit::SignedVoluntaryExit,
    },
    hash::{H256, H768},
    ibc::lightclients::ethereum::sync_aggregate::UnboundedSyncAggregate,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody>
#[cfg(feature = "ssz")]
#[model]
#[derive(Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct BeaconBlockBody<
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
    pub attester_slashings: List<AttesterSlashing<C>, C::MAX_ATTESTER_SLASHINGS>,
    pub attestations: List<Attestation<C>, C::MAX_ATTESTATIONS>,
    pub deposits: List<Deposit<C>, C::MAX_DEPOSITS>,
    pub voluntary_exits: List<SignedVoluntaryExit, C::MAX_VOLUNTARY_EXITS>,
    pub sync_aggregate: SyncAggregate<C>,
    pub execution_payload: ExecutionPayload<C>,
    pub bls_to_execution_changes: List<SignedBlsToExecutionChange, C::MAX_BLS_TO_EXECUTION_CHANGES>,
    pub blob_kzg_commitments: List<KZGCommitment, C::MAX_BLOB_COMMITMENTS_PER_BLOCK>,
}

#[model]
pub struct UnboundedBeaconBlockBody {
    pub randao_reveal: H768,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: Vec<ProposerSlashing>,
    pub attester_slashings: Vec<UnboundedAttesterSlashing>,
    pub attestations: Vec<UnboundedAttestation>,
    pub deposits: Vec<UnboundedDeposit>,
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
    pub sync_aggregate: UnboundedSyncAggregate,
    pub execution_payload: UnboundedExecutionPayload,
    pub bls_to_execution_changes: Vec<SignedBlsToExecutionChange>,
    pub blob_kzg_commitments: Vec<KZGCommitment>,
}
