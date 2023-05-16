use crate::internal_prelude::*;
use displaydoc::Display;
use ethereum_consensus::{
    beacon::{BeaconBlockHeader, Root, Slot},
    bls::PublicKey,
    errors::MerkleError,
    sync_protocol::SyncCommitteePeriod,
};
use trie_db::TrieError;

type BoxedTrieError = Box<TrieError<primitive_types::H256, rlp::DecoderError>>;

#[derive(Debug, Display)]
pub enum Error {
    /// invalid signature period: `store={0} signature={1} reason={2}`
    InvalidSingaturePeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// invalid finalized period: `store={0} finalized={1} reason={2}`
    InvalidFinalizedPeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// not finalized period: `finalized={0} attested={1}`
    NotFinalizedUpdate(SyncCommitteePeriod, SyncCommitteePeriod),
    /// cannot rotate to next sync committee: `store={0} finalized={1}`
    CannotRotateNextSyncCommittee(SyncCommitteePeriod, SyncCommitteePeriod),
    /// verify membership error
    VerifyMembershipError(),
    /// trusted root mismatch: `expected={0:?} actual={1:?}`
    TrustedRootMismatch(Root, Root),
    /// less than the minimal participants' `actual={0} minimal={1}`
    LessThanMinimalParticipants(usize, usize),
    /// insufficient participants: `actual={0} total={1}`
    InsufficientParticipants(u64, u64),
    /// invalid bls signatures
    InvalidBLSSignatures,
    /// finalized header not found
    FinalizedHeaderNotFound,
    /// inconsistent slot order: `current={0} signature={1} attested={2} finalized={3}`
    InconsistentSlotOrder(Slot, Slot, Slot, Slot),
    /// irrelevant consensus updates error: `{0}`
    IrrelevantConsensusUpdates(String),
    /// trie error
    TrieError(BoxedTrieError),
    /// ethereum common error: `{0:?}`
    CommonError(ethereum_consensus::errors::Error),
    /// rlp decoder error: `{0:?}`
    RlpDecoderError(rlp::DecoderError),
    /// misbehaviour error: `{0}`
    Misbehaviour(MisbehaviourError),
    /// both updates of misbehaviour data must have same period: {0} != {1}
    DifferentPeriodInNextSyncCommitteeMisbehaviour(SyncCommitteePeriod, SyncCommitteePeriod),
    /// both updates of misbehaviour data must have next sync committee
    NoNextSyncCommitteeInNextSyncCommitteeMisbehaviour,
    /// TODO
    NoNextSyncCommittee,
    /// both updates of misbehaviour data must have different next sync committee: aggregate_pubkey={0:?}
    SameNextSyncCommitteeInNextSyncCommitteeMisbehaviour(PublicKey),
    /// both updates of misbehaviour data must have same finalized slot: {0} != {1}
    DifferentSlotInFinalizedHeaderMisbehaviour(Slot, Slot),
    /// both updates of misbehaviour data must have different finalized header: {0:?}
    SameFinalizedHeaderInFinalizedHeaderMisbehaviour(BeaconBlockHeader),
    /// non-existence error in execution layer
    ExecutionValueNonExist,
    /// existence error in execution layer
    ExecutionValueExist,
    /// value mismatch error in execution layer: {0:?} != {1:?}
    ExecutionValueMismatch(Vec<u8>, Vec<u8>),
    /// invalid merkle branch of finalized beacon header: `error={0}`
    InvalidFinalizedBeaconHeaderMerkleBranch(MerkleError),
    /// invalid merkle branch of finalized execution payload: `error={0}`
    InvalidFinalizedExecutionPayload(MerkleError),
    /// invalid merkle branch of next sync committee: `error={0}`
    InvalidNextSyncCommitteeMerkleBranch(MerkleError),
    /// invalid merkle branch of current sync committee: `error={0}`
    InvalidCurrentSyncCommitteeMerkleBranch(MerkleError),
    /// invalid merkle branch of execution state root: `error={0}`
    InvalidExecutionStateRootMerkleBranch(MerkleError),
    /// invalid merkle branch of execution block number: `error={0}`
    InvalidExecutionBlockNumberMerkleBranch(MerkleError),
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<BoxedTrieError> for Error {
    fn from(value: BoxedTrieError) -> Self {
        Self::TrieError(value)
    }
}

impl From<ethereum_consensus::errors::Error> for Error {
    fn from(value: ethereum_consensus::errors::Error) -> Self {
        Self::CommonError(value)
    }
}

impl From<rlp::DecoderError> for Error {
    fn from(value: rlp::DecoderError) -> Self {
        Self::RlpDecoderError(value)
    }
}

#[derive(Debug, Display)]
pub enum MisbehaviourError {
    /// next sync committee: `{0:?} != {1:?}`
    InconsistentNextSyncCommittee(PublicKey, PublicKey),
}

#[cfg(feature = "std")]
impl std::error::Error for MisbehaviourError {}

impl From<MisbehaviourError> for Error {
    fn from(value: MisbehaviourError) -> Self {
        Self::Misbehaviour(value)
    }
}
