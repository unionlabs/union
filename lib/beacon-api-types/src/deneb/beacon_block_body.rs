use unionlabs::primitives::{FixedBytes, H256, H768};
#[cfg(feature = "ssz")]
use {
    crate::{
        altair::SyncAggregateSsz,
        chain_spec::ChainSpec,
        deneb::ExecutionPayloadSsz,
        phase0::{AttestationSsz, AttesterSlashingSsz, DepositSsz},
    },
    ssz::{types::List, Ssz},
};

use crate::{
    altair::SyncAggregate,
    capella::SignedBlsToExecutionChange,
    deneb::ExecutionPayload,
    phase0::{
        Attestation, AttesterSlashing, Deposit, Eth1Data, ProposerSlashing, SignedVoluntaryExit,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
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
    pub blob_kzg_commitments: Vec<FixedBytes<48>>,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct BeaconBlockBodySsz<C: ChainSpec> {
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
    pub blob_kzg_commitments: List<FixedBytes<48>, C::MAX_BLOB_COMMITMENTS_PER_BLOCK>,
}
