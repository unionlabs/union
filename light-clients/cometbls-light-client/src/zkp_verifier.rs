use unionlabs::{hash::H256, ibc::lightclients::cometbls::light_header::LightHeader};

pub trait ZkpVerifier {
    fn verify_zkp(
        chain_id: &str,
        trusted_validators_hash: H256,
        header: &LightHeader,
        zkp: &[u8],
    ) -> Result<(), cometbls_groth16_verifier::Error> {
        cometbls_groth16_verifier::verify_zkp(chain_id, trusted_validators_hash, header, zkp)
    }
}

impl ZkpVerifier for () {}

pub struct MockZKPVerifier;

impl ZkpVerifier for MockZKPVerifier {
    fn verify_zkp(
        _chain_id: &str,
        _trusted_validators_hash: H256,
        _header: &LightHeader,
        _zkp: &[u8],
    ) -> Result<(), cometbls_groth16_verifier::Error> {
        Ok(())
    }
}
