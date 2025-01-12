use beacon_api_types::Slot;
use milagro_bls::AmclError;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    primitives::{H256, H384},
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
        update_attested_slot: Slot,
        trusted_finalized_slot: Slot,
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
        update_signature_slot: Slot,
        update_attested_slot: Slot,
        update_finalized_slot: Slot,
    },
    #[error(
        "update slot {update_signature_slot} is more recent than the \
        calculated current slot {current_slot}"
    )]
    UpdateMoreRecentThanCurrentSlot {
        current_slot: Slot,
        update_signature_slot: Slot,
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
    NextSyncCommitteeMismatch { expected: H384, found: H384 },
    #[error("insufficient number of sync committee participants ({0})")]
    InsufficientSyncCommitteeParticipants(usize),
    #[error("bls error ({0:?})")]
    Bls(AmclError),
    // boxed as this variant is significantly larger than the rest of the variants (due to the BlsSignature contained within)
    #[error(transparent)]
    InvalidSignature(Box<InvalidSignature>),
    #[error("update header contains deneb specific information")]
    MustBeDeneb,
    #[error("finalized slot cannot be the genesis slot")]
    FinalizedSlotIsGenesis,
    #[error("client errored during signature verification ({0})")]
    ClientSignatureVerification(String),
}

// NOTE: Implemented here instead of via #[from] since AmclError doesn't implement core::error::Error
impl From<AmclError> for Error {
    fn from(e: AmclError) -> Self {
        Error::Bls(e)
    }
}
