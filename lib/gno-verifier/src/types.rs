use gno_types::PublicKey;

pub trait SignatureVerifier {
    fn verify_signature(&self, pub_key: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;
}
