#[deprecated(note = "the circuit has been generalized, use verify_zkp_v2() instead")]
pub fn verify_zkp(
    trusted_validators_hash: &[u8],
    untrusted_validators_hash: &[u8],
    message: &[u8],
    zkp: &[u8],
) -> bool {
    cometbls_groth16_verifier::verify_zkp(
        cometbls_groth16_verifier::testnet_vk(),
        trusted_validators_hash.into(),
        untrusted_validators_hash.into(),
        message,
        zkp,
    )
    .map_or(false, |_| true)
}

pub fn verify_zkp_v2(
    trusted_validators_hash: &[u8],
    untrusted_validators_hash: &[u8],
    message: &[u8],
    zkp: &[u8],
) -> bool {
    cometbls_groth16_verifier::verify_zkp_v2(
        trusted_validators_hash.into(),
        untrusted_validators_hash.into(),
        message,
        zkp,
    )
    .map_or(false, |_| true)
}
