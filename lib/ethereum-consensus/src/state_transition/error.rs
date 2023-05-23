use crate::crypto::Error as CryptoError;
use crate::phase0::{AttestationData, BeaconBlockHeader, Checkpoint};
use crate::primitives::{BlsSignature, Bytes32, Epoch, Hash32, Root, Slot, ValidatorIndex};
use crate::state_transition::Forks;
use ssz_rs::prelude::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Merkleization(#[from] MerkleizationError),
    #[error("{0}")]
    SimpleSerialize(#[from] SimpleSerializeError),
    #[error("{0}")]
    Crypto(#[from] CryptoError),
    #[cfg(feature = "serde")]
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "serde")]
    #[error("{0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("requested element {requested} but collection only has {bound} elements")]
    OutOfBounds { requested: usize, bound: usize },
    #[error("collection cannot be empty")]
    CollectionCannotBeEmpty,
    #[error("given index {index} is greater than the total amount of indices {total}")]
    InvalidShufflingIndex { index: usize, total: usize },
    #[error("slot {requested} is outside of allowed range ({lower_bound}, {upper_bound})")]
    SlotOutOfRange {
        requested: Slot,
        lower_bound: Slot,
        upper_bound: Slot,
    },
    #[error("overflow")]
    Overflow,
    #[error("underflow")]
    Underflow,
    #[error("{0}")]
    InvalidBlock(#[from] Box<InvalidBlock>),
    #[error("an invalid transition to a past slot {requested} from slot {current}")]
    TransitionToPreviousSlot { current: Slot, requested: Slot },
    #[error("invalid state root")]
    InvalidStateRoot,
    #[error(
    "the requested epoch {requested} is not in the required current epoch {current} or previous epoch {previous}"
    )]
    InvalidEpoch {
        requested: Epoch,
        previous: Epoch,
        current: Epoch,
    },
    #[error(
        "transition requested from a later fork {destination_fork:?} to an earlier fork {source_fork:?}"
    )]
    IncompatibleForks {
        source_fork: Forks,
        destination_fork: Forks,
    },
    #[error("genesis time unknown for network {0}")]
    UnknownGenesisTime(String),
    #[cfg(feature = "serde")]
    #[error("an unknown preset {0} was supplied when constructing context")]
    UnknownPreset(String),
}

#[derive(Debug, Error)]
pub enum InvalidBlock {
    #[error("invalid beacon block header: {0}")]
    Header(#[from] InvalidBeaconBlockHeader),
    #[error("invalid operation: {0}")]
    InvalidOperation(#[from] InvalidOperation),
}

#[derive(Debug, Error)]
pub enum InvalidOperation {
    #[error("invalid attestation: {0}")]
    Attestation(#[from] InvalidAttestation),
    #[error("invalid indexed attestation: {0}")]
    IndexedAttestation(#[from] InvalidIndexedAttestation),
    #[error("invalid deposit: {0}")]
    Deposit(#[from] InvalidDeposit),
    #[error("invalid randao (Bls signature): {0:?}")]
    Randao(BlsSignature),
    #[error("invalid proposer slashing: {0}")]
    ProposerSlashing(#[from] InvalidProposerSlashing),
    #[error("invalid attester slashing: {0}")]
    AttesterSlashing(#[from] InvalidAttesterSlashing),
    #[error("invalid voluntary exit: {0}")]
    VoluntaryExit(#[from] InvalidVoluntaryExit),
    #[error("invalid sync aggregate: {0}")]
    SyncAggregate(#[from] InvalidSyncAggregate),
    #[error("invalid execution payload: {0}")]
    ExecutionPayload(#[from] InvalidExecutionPayload),
}

#[derive(Debug, Error)]
pub enum InvalidBeaconBlockHeader {
    #[error("mismatch between state slot {state_slot} and block slot {block_slot}")]
    StateSlotMismatch { state_slot: Slot, block_slot: Slot },
    #[error("mismatch between the block's parent root {expected:?} and the expected parent root {provided:?}")]
    ParentBlockRootMismatch { expected: Root, provided: Root },
    #[error("proposer with index {0} is slashed")]
    ProposerSlashed(ValidatorIndex),
    #[error("block slot {block_slot} is older than the latest block header slot {latest_block_header_slot}")]
    OlderThanLatestBlockHeader {
        block_slot: Slot,
        latest_block_header_slot: Slot,
    },
    #[error("mismatch between the block proposer index {block_proposer_index} and the state proposer index {proposer_index}")]
    ProposerIndexMismatch {
        block_proposer_index: ValidatorIndex,
        proposer_index: ValidatorIndex,
    },
}

#[derive(Debug, Error)]
pub enum InvalidAttestation {
    #[error("expected length of {expected_length} in bitfield but had length {length}")]
    Bitfield {
        expected_length: usize,
        length: usize,
    },
    #[error("invalid target epoch {target}, not current ({current}) or previous epochs")]
    InvalidTargetEpoch { target: Epoch, current: Epoch },
    #[error("invalid slot {slot} (in epoch {epoch}) based on target epoch {target}")]
    InvalidSlot {
        slot: Slot,
        epoch: Epoch,
        target: Epoch,
    },
    #[error("attestation at slot {attestation_slot} is not timely for state slot {state_slot}, outside of range [{lower_bound}, {upper_bound}]")]
    NotTimely {
        state_slot: Slot,
        attestation_slot: Slot,
        lower_bound: Slot,
        upper_bound: Slot,
    },
    #[error("attestation's index {index} exceeds the current committee count {upper_bound}")]
    InvalidIndex { index: usize, upper_bound: usize },
    #[error("attestation's source checkpoint {source_checkpoint:?} does not match the expected checkpoint {expected:?} (in epoch {current})")]
    InvalidSource {
        expected: Checkpoint,
        source_checkpoint: Checkpoint,
        current: Epoch,
    },
}

#[derive(Debug, Error)]
pub enum InvalidIndexedAttestation {
    #[error("attesting indices are empty")]
    AttestingIndicesEmpty,
    #[error("attesting indices are duplicated")]
    DuplicateIndices(Vec<ValidatorIndex>),
    #[error("attesting indices are not sorted")]
    AttestingIndicesNotSorted,
    #[error("index in attesting set is invalid for this state")]
    InvalidIndex(ValidatorIndex),
}

#[derive(Debug, Error)]
pub enum InvalidDeposit {
    #[error("expected {expected} deposits but only had {count} deposits")]
    IncorrectCount { expected: usize, count: usize },
    #[error("merkle validation failed for tree with depth {depth} and root {root:?} at index {index} for leaf {leaf:?} and branch {branch:?}")]
    InvalidProof {
        leaf: Node,
        branch: Vec<Node>,
        depth: usize,
        index: usize,
        root: Root,
    },
    #[error("invalid signature for deposit: {0:?}")]
    InvalidSignature(BlsSignature),
}

#[derive(Debug, Error)]
pub enum InvalidProposerSlashing {
    #[error("different slots: {0} vs. {1}")]
    SlotMismatch(Slot, Slot),
    #[error("different proposers: {0} vs. {1}")]
    ProposerMismatch(ValidatorIndex, ValidatorIndex),
    #[error("headers are equal: {0:?}")]
    HeadersAreEqual(BeaconBlockHeader),
    #[error("proposer with index {0} is not slashable")]
    ProposerIsNotSlashable(ValidatorIndex),
    #[error("header has invalid signature: {0:?}")]
    InvalidSignature(BlsSignature),
    #[error("proposer with index {0} is not in state")]
    InvalidIndex(ValidatorIndex),
}

#[derive(Debug, Error)]
pub enum InvalidAttesterSlashing {
    #[error("attestation data is not slashable: {0:?} vs. {1:?}")]
    NotSlashable(Box<AttestationData>, Box<AttestationData>),
    #[error("no validator was slashed across indices: {0:?}")]
    NoSlashings(Vec<ValidatorIndex>),
}

#[derive(Debug, Error)]
pub enum InvalidVoluntaryExit {
    #[error("validator with index {0} is not in state")]
    InvalidIndex(ValidatorIndex),
    #[error("validator is not active in the current epoch {0}")]
    InactiveValidator(Epoch),
    #[error("validator {index} already exited in {epoch}")]
    ValidatorAlreadyExited { index: ValidatorIndex, epoch: Epoch },
    #[error("exit in epoch {exit_epoch} is not eligible for processing in current epoch {current_epoch}")]
    EarlyExit {
        current_epoch: Epoch,
        exit_epoch: Epoch,
    },
    #[error("validator needs to be active for a minimum period of time (from epoch {minimum_time_active}, currently in {current_epoch})")]
    ValidatorIsNotActiveForLongEnough {
        current_epoch: Epoch,
        minimum_time_active: Epoch,
    },
    #[error("voluntary exit has invalid signature: {0:?}")]
    InvalidSignature(BlsSignature),
}

#[derive(Debug, Error)]
pub enum InvalidSyncAggregate {
    #[error("invalid sync committee aggregate signature {signature} signing over previous slot block root {root}")]
    InvalidSignature { signature: BlsSignature, root: Root },
}

#[derive(Debug, Error)]
pub enum InvalidExecutionPayload {
    #[error("expected parent hash {expected} but block has parent hash {provided}")]
    InvalidParentHash { provided: Hash32, expected: Hash32 },
    #[error("expected randao value {expected} but block has randao value {provided}")]
    InvalidPrevRandao {
        provided: Bytes32,
        expected: Bytes32,
    },
    #[error("expected timestamp {expected} but block has timestamp {provided}")]
    InvalidTimestamp { provided: u64, expected: u64 },
}

pub(crate) fn invalid_header_error(error: InvalidBeaconBlockHeader) -> Error {
    Error::InvalidBlock(Box::new(InvalidBlock::Header(error)))
}

pub(crate) fn invalid_operation_error(error: InvalidOperation) -> Error {
    Error::InvalidBlock(Box::new(InvalidBlock::InvalidOperation(error)))
}
