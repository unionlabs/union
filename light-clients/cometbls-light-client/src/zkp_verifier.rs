use unionlabs::{hash::H256, uint::U256};

pub trait ZKPVerifier {
    fn verify_zkp(
        trusted_validators_hash: H256,
        untrusted_validators_hash: H256,
        message: &[u8],
        zkp: &[u8],
    ) -> Result<(), cometbls_groth16_verifier::Error> {
        cometbls_groth16_verifier::verify_zkp(
            U256::from_big_endian(trusted_validators_hash.0),
            U256::from_big_endian(untrusted_validators_hash.0),
            message,
            zkp,
        )
    }
}

impl ZKPVerifier for () {}

pub struct MockZKPVerifier;

impl ZKPVerifier for MockZKPVerifier {
    fn verify_zkp(
        _trusted_validators_hash: H256,
        _untrusted_validators_hash: H256,
        _message: &[u8],
        _zkp: &[u8],
    ) -> Result<(), cometbls_groth16_verifier::Error> {
        Ok(())
    }
}
