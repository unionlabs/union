/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
pub mod attestation;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
pub mod attestation_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
pub mod attester_slashing;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
pub mod beacon_block;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody>
pub mod beacon_block_body;
pub mod beacon_block_header;
pub mod bls_to_execution_change;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
pub mod checkpoint;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
pub mod deposit;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
pub mod deposit_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
pub mod eth1_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayload>
pub mod execution_payload;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayloadheader>
pub mod execution_payload_header;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#fork>
pub mod fork;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#forkdata>
pub mod fork_data;
pub mod fork_parameters;
pub mod genesis_data;
pub mod indexed_attestation;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
pub mod light_client_bootstrap;
pub mod light_client_finality_update;
pub mod light_client_header;
/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#lightclientupdate>
pub mod light_client_update;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
pub mod proposer_slashing;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock>
pub mod signed_beacon_block;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
pub mod signed_beacon_block_header;
pub mod signed_bls_to_execution_change;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
pub mod signed_voluntary_exit;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signingdata>
pub mod signing_data;
pub mod sync_aggregate;
pub mod sync_committee;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
pub mod voluntary_exit;
pub mod withdrawal;

/// Newtype for beacon slots.
pub mod slot;

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
pub mod custom_types;

pub mod chain_spec;

/// Values that are constant across all configurations.
pub mod consts;

pub mod preset;

pub use crate::{
    attestation::Attestation, attestation_data::AttestationData,
    attester_slashing::AttesterSlashing, beacon_block::BeaconBlock,
    beacon_block_body::BeaconBlockBody, beacon_block_header::BeaconBlockHeader,
    bls_to_execution_change::BlsToExecutionChange, chain_spec::*, checkpoint::Checkpoint,
    deposit::Deposit, deposit_data::DepositData, eth1_data::Eth1Data,
    execution_payload::ExecutionPayload, fork::Fork, fork_data::ForkData,
    fork_parameters::ForkParameters, genesis_data::GenesisData,
    indexed_attestation::IndexedAttestation, light_client_bootstrap::LightClientBootstrap,
    light_client_finality_update::LightClientFinalityUpdate,
    light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
    proposer_slashing::ProposerSlashing, signed_beacon_block::SignedBeaconBlock,
    signed_beacon_block_header::SignedBeaconBlockHeader,
    signed_bls_to_execution_change::SignedBlsToExecutionChange,
    signed_voluntary_exit::SignedVoluntaryExit, signing_data::SigningData,
    sync_aggregate::SyncAggregate, sync_committee::SyncCommittee, voluntary_exit::VoluntaryExit,
    withdrawal::Withdrawal,
};
#[cfg(feature = "ssz")]
pub use crate::{
    attestation::AttestationSsz, attester_slashing::AttesterSlashingSsz,
    beacon_block::BeaconBlockSsz, beacon_block_body::BeaconBlockBodySsz, deposit::DepositSsz,
    execution_payload::ExecutionPayloadSsz, execution_payload_header::ExecutionPayloadHeaderSsz,
    indexed_attestation::IndexedAttestationSsz, light_client_header::LightClientHeaderSsz,
    signed_beacon_block::SignedBeaconBlockSsz, sync_aggregate::SyncAggregateSsz,
    sync_committee::SyncCommitteeSsz,
};
