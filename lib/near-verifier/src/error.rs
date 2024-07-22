use near_primitives_core::hash::CryptoHash;

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
    #[error("state proof verification failed")]
    StateVerificationFailure,
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
