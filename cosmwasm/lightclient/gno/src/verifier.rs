use cosmwasm_std::Deps;
use gno_types::PublicKey;
use gno_verifier::types::SignatureVerifier;

pub struct CwVerifier<'a> {
    deps: Deps<'a>,
}

impl<'a> CwVerifier<'a> {
    pub fn new(deps: Deps<'a>) -> Self {
        Self { deps }
    }
}

#[allow(clippy::manual_unwrap_or)]
impl SignatureVerifier for CwVerifier<'_> {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
        match pubkey {
            PublicKey::Ed25519(key) => self
                .deps
                .api
                .ed25519_verify(msg, sig, key)
                .unwrap_or_default(),
            PublicKey::Secp256k1(key) => self
                .deps
                .api
                .secp256k1_verify(msg, sig, key)
                .unwrap_or_default(),
            PublicKey::Multisig { .. } => false,
        }
    }
}
