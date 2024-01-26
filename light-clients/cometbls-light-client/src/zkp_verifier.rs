pub trait ZKPVerifier {
    fn verify_zkp(
        trusted_validators_hash: &[u8],
        untrusted_validators_hash: &[u8],
        message: &[u8],
        zkp: &[u8],
    ) -> bool {
        cometbls_groth16_verifier::verify_zkp(
            trusted_validators_hash.into(),
            untrusted_validators_hash.into(),
            message,
            zkp,
        )
        .map_or(false, |_| true)
    }
}

impl ZKPVerifier for () {}

pub struct MockZKPVerifier;

impl ZKPVerifier for MockZKPVerifier {
    fn verify_zkp(
        _trusted_validators_hash: &[u8],
        _untrusted_validators_hash: &[u8],
        _message: &[u8],
        _zkp: &[u8],
    ) -> bool {
        true
    }
}
