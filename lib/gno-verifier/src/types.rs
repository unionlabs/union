use gno_types::PublicKey;

pub trait HostFns {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;
}
