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
}
