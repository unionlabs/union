use unionlabs::tendermint::crypto::public_key::PublicKey;

pub trait HostFns {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;

    fn verify_batch_signature(&self, pubkeys: &[PublicKey], msgs: &[&[u8]], sigs: &[&[u8]])
        -> bool;
}

pub struct SignatureVerifier<V: HostFns> {
    pub verifier: V,
}

impl<V: HostFns> SignatureVerifier<V> {
    pub fn new(verifier: V) -> Self {
        Self { verifier }
    }
}
