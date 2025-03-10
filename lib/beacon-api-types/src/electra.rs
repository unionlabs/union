pub use crate::electra::{
    attestation::Attestation, attester_slashing::AttesterSlashing, beacon_block::BeaconBlock,
    beacon_block_body::BeaconBlockBody, consolidation_request::ConsolidationRequest,
    deposit_request::DepositRequest, execution_requests::ExecutionRequests,
    indexed_attestation::IndexedAttestation, light_client_bootstrap::LightClientBootstrap,
    light_client_finality_update::LightClientFinalityUpdate,
    light_client_update::LightClientUpdate, signed_beacon_block::SignedBeaconBlock,
    withdrawal_request::WithdrawalRequest,
};
#[cfg(feature = "ssz")]
pub use crate::electra::{
    attestation::AttestationSsz, attester_slashing::AttesterSlashingSsz,
    beacon_block::BeaconBlockSsz, beacon_block_body::BeaconBlockBodySsz,
    execution_requests::ExecutionRequestsSsz, indexed_attestation::IndexedAttestationSsz,
    signed_beacon_block::SignedBeaconBlockSsz,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#attestation>
pub mod attestation;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#attesterslashing>
pub mod attester_slashing;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#indexedattestation>
pub mod indexed_attestation;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#beaconblockbody>
pub mod beacon_block_body;

/// Updated indirectly.
pub mod beacon_block;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#executionrequests>
pub mod execution_requests;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#consolidationrequest>
pub mod consolidation_request;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#depositrequest>
pub mod deposit_request;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/beacon-chain.md#withdrawalrequest>
pub mod withdrawal_request;

/// Updated indirectly.
pub mod signed_beacon_block;

/// Changed due to <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#new-constants>.
pub mod light_client_update;

/// Changed due to <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#new-constants>.
pub mod light_client_finality_update;

/// Changed due to <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#new-constants>.
pub mod light_client_bootstrap;
