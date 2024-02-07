use std::fmt::Display;

use unionlabs::tendermint::crypto::public_key::PublicKey;

pub trait BatchSignatureVerifier {
    type Error: 'static + std::error::Error;
    /// Implementer should decide whether it's going to make sense to
    /// do batch verification based on how many signatures we have.
    fn should_batch_verify(signature_len: usize) -> bool;

    fn new() -> Self;

    fn add(
        &mut self,
        pubkey: &PublicKey,
        msg: Vec<u8>,
        signature: &[u8],
    ) -> Result<(), Self::Error>;

    fn verify_signature(&self) -> bool;
}

#[derive(Debug)]
pub struct BatchVerificationError;

impl Display for BatchVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BatchVerificationError {}

impl BatchSignatureVerifier for () {
    type Error = BatchVerificationError;

    fn should_batch_verify(_signature_len: usize) -> bool {
        false
    }

    fn new() -> Self {
        ()
    }

    fn add(
        &mut self,
        _pubkey: &PublicKey,
        _msg: Vec<u8>,
        _signature: &[u8],
    ) -> Result<(), Self::Error> {
        Err(BatchVerificationError)
    }

    fn verify_signature(&self) -> bool {
        false
    }
}

pub trait SignatureVerifier {
    fn verify_signature(pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;
}
