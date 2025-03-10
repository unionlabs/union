pub use crate::phase0::{
    attestation::Attestation, attestation_data::AttestationData,
    attester_slashing::AttesterSlashing, beacon_block::BeaconBlock,
    beacon_block_body::BeaconBlockBody, beacon_block_header::BeaconBlockHeader,
    checkpoint::Checkpoint, deposit::Deposit, deposit_data::DepositData, eth1_data::Eth1Data,
    fork::Fork, fork_data::ForkData, indexed_attestation::IndexedAttestation,
    proposer_slashing::ProposerSlashing, signed_beacon_block::SignedBeaconBlock,
    signed_beacon_block_header::SignedBeaconBlockHeader,
    signed_voluntary_exit::SignedVoluntaryExit, signing_data::SigningData,
    voluntary_exit::VoluntaryExit,
};
#[cfg(feature = "ssz")]
pub use crate::phase0::{
    attestation::AttestationSsz, attester_slashing::AttesterSlashingSsz,
    beacon_block::BeaconBlockSsz, beacon_block_body::BeaconBlockBodySsz, deposit::DepositSsz,
    indexed_attestation::IndexedAttestationSsz, signed_beacon_block::SignedBeaconBlockSsz,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
pub mod attestation;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
pub mod attestation_data;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
pub mod attester_slashing;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
pub mod beacon_block;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblockbody>
pub mod beacon_block_body;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblockheader>
pub mod beacon_block_header;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
pub mod checkpoint;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
pub mod deposit;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
pub mod deposit_data;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
pub mod eth1_data;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#fork>
pub mod fork;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#forkdata>
pub mod fork_data;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#indexedattestation>
pub mod indexed_attestation;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
pub mod proposer_slashing;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock>
pub mod signed_beacon_block;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
pub mod signed_beacon_block_header;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
pub mod signed_voluntary_exit;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signingdata>
pub mod signing_data;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
pub mod voluntary_exit;
