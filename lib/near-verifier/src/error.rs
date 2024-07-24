use near_account_id::AccountId;
use near_primitives_core::hash::CryptoHash;
use unionlabs::near::raw_state_proof::RawStateProof;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("epoch block producers not found for ({0:?})")]
    EpochBlockProducersNotFound(CryptoHash),
    #[error("public key must be of type `ed25519`")]
    UnsupportedPublicKey,
    #[error("signature type must be `ed25519`")]
    UnsupportedSignature,
    #[error("signature verification failed for pubkey {0:?} signature {1:?} message {2:?}")]
    VerificationFailure(Vec<u8>, Vec<u8>, Vec<u8>),
    // TODO(aeryz): add context
    #[error("merkle verification failed")]
    MerkleVerificationFailure,
    #[error(
        "state proof verification failed. Params: {0:?}\n\n{1:?}\n\n{2:?}\n\n{3:?}\n\n{4:?}\n\n"
    )]
    StateVerificationFailure(Vec<u8>, Vec<u8>, AccountId, Vec<u8>, Vec<u8>),
    #[error("update height ({0}) must be greater than the current height ({1})")]
    UpdateHeightMustBeGreater(u64, u64),
    #[error("can only be updated within the same epoch or the next epoch, but got ({0:?})")]
    InvalidEpochId(CryptoHash),
    #[error("when updating to the next epoch, `next_bps` must be provided")]
    MustHaveNextEpochId,
    #[error("approved stake ({0}) is below the threshold ({1})")]
    ApprovedStakeBelowThreshold(u128, u128),
    #[error("next bp hash mismatch ({0} != {1})")]
    NextBpsHashMismatch(CryptoHash, CryptoHash),
}
