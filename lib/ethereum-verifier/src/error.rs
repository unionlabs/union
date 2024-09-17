use milagro_bls::AmclError;
use trie_db::TrieError;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    hash::H256,
};

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
#[error("invalid merkle branch \
    (leaf: {leaf}, branch: [{branch}], \
    depth: {depth}, index: {index}, root: {root})",
    branch = .branch.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
)]
pub struct InvalidMerkleBranch {
    pub leaf: H256,
    pub branch: Vec<H256>,
    pub depth: usize,
    pub index: u64,
    pub root: H256,
}

#[derive(Debug, PartialEq, thiserror::Error, Clone)]
#[error("signature cannot be verified (public_keys: {public_keys:?}, msg: {msg}, signature: {signature})", msg = serde_utils::to_hex(.msg))]
pub struct InvalidSignature {
    pub public_keys: Vec<BlsPublicKey>,
    pub msg: Vec<u8>,
    pub signature: BlsSignature,
}

#[derive(Debug, PartialEq, thiserror::Error, Clone)]
pub enum Error {
    #[error(transparent)]
    InvalidMerkleBranch(#[from] InvalidMerkleBranch),
    #[error("invalid chain version")]
    InvalidChainVersion,
    #[error("crypto error")]
    Crypto,
    #[error(
        "expected current sync committee to be provided since `update_period == current_period`"
    )]
    ExpectedCurrentSyncCommittee,
    #[error("expected next sync committee to be provided since `update_period > current_period`")]
    ExpectedNextSyncCommittee,
    #[error(
        "irrelevant update since the order of the slots in the update data, and stored data is not correct. \
        either the update_attested_slot (found {update_attested_slot}) must be > the trusted_finalized_slot \
        (found {trusted_finalized_slot}) or if it is not, then the update_attested_period \
        (found {update_attested_period}) must be the same as the store_period (found {stored_period}) and \
        the update_sync_committee must be set (was set: {update_sync_committee_is_set}) and the trusted \
        next_sync_committee must be unset (was set: {trusted_next_sync_committee_is_set})"
    )]
    IrrelevantUpdate {
        update_attested_slot: u64,
        trusted_finalized_slot: u64,
        update_attested_period: u64,
        stored_period: u64,
        update_sync_committee_is_set: bool,
        trusted_next_sync_committee_is_set: bool,
    },
    #[error(
        "(update_signature_slot > update_attested_slot >= update_finalized_slot) must hold, \
        found: ({update_signature_slot} > {update_attested_slot} >= {update_finalized_slot})"
    )]
    InvalidSlots {
        update_signature_slot: u64,
        update_attested_slot: u64,
        update_finalized_slot: u64,
    },
    #[error(
        "update slot {update_signature_slot} is more recent than the \
        calculated current slot {current_slot}"
    )]
    UpdateMoreRecentThanCurrentSlot {
        current_slot: u64,
        update_signature_slot: u64,
    },
    #[error(
        "signature period ({signature_period}) must be equal to `store_period` \
        ({stored_period}) or `store_period + 1` when the next sync committee is stored"
    )]
    InvalidSignaturePeriodWhenNextSyncCommitteeExists {
        signature_period: u64,
        stored_period: u64,
    },
    #[error(
        "signature period ({signature_period}) must be equal to `store_period` \
        ({stored_period}) when the next sync committee is not stored"
    )]
    InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist {
        signature_period: u64,
        stored_period: u64,
    },
    #[error(
        "next sync committee ({found}) does not match with the one in the current state ({expected})"
    )]
    NextSyncCommitteeMismatch {
        expected: BlsPublicKey,
        found: BlsPublicKey,
    },
    #[error("insufficient number of sync committee participants ({0})")]
    InsufficientSyncCommitteeParticipants(usize),
    #[error("bls error ({0:?})")]
    Bls(AmclError),
    #[error(
        "proof is invalid due to value mismatch, expected: {expected}, actual: {actual}",
        expected = serde_utils::to_hex(expected),
        actual = serde_utils::to_hex(actual)
    )]
    ValueMismatch { expected: Vec<u8>, actual: Vec<u8> },
    #[error("proof is invalid due to missing value: {v}", v = serde_utils::to_hex(value))]
    ValueMissing { value: Vec<u8> },
    #[error("trie error ({0:?})")]
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    // we us debug here because the display implementation for rlp::DecoderError is stupid
    #[error("rlp decoding failed: {0:?}")]
    RlpDecode(#[from] rlp::DecoderError),
    #[error("custom query error: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),
    // boxed as this variant is significantly larger than the rest of the variants (due to the BlsSignature contained within)
    #[error(transparent)]
    InvalidSignature(Box<InvalidSignature>),
    #[error("update header contains deneb specific information")]
    MustBeDeneb,
    #[error("finalized slot cannot be the genesis slot")]
    FinalizedSlotIsGenesis,
}

// NOTE: Implemented here instead of via #[from] since AmclError doesn't implement std::error::Error
impl From<AmclError> for Error {
    fn from(e: AmclError) -> Self {
        Error::Bls(e)
    }
}

// NOTE: Implemented here instead of via #[from] since Box<TrieError<primitive_types::H256, rlp::DecoderError>> doesn't implement std::error::Error
impl From<Box<TrieError<primitive_types::H256, rlp::DecoderError>>> for Error {
    fn from(e: Box<TrieError<primitive_types::H256, rlp::DecoderError>>) -> Self {
        Error::Trie(e)
    }
}
