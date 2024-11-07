// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::groth16;

#[allow(unused_const)]
// Error for input is not a valid Arkwork representation of a verifying key.
const EInvalidVerifyingKey: u64 = 0;

#[allow(unused_const)]
// Error if the given curve is not supported
const EInvalidCurve: u64 = 1;

// Error if the number of public inputs given exceeds the max.
const ETooManyPublicInputs: u64 = 2;

// Error a public input does not have the correct length.
const EInvalidScalar: u64 = 3;

// We need to set an upper bound on the number of public inputs to avoid a DoS attack.
const MaxPublicInputs: u64 = 8; // This must match the corresponding constant in the native verify function.

/// Represents an elliptic curve construction to be used in the verifier. Currently we support BLS12-381 and BN254.
/// This should be given as the first parameter to `prepare_verifying_key` or `verify_groth16_proof`.
public struct Curve has store, copy, drop {
    id: u8,
}

/// Return the `Curve` value indicating that the BLS12-381 construction should be used in a given function.
public fun bls12381(): Curve { Curve { id: 0 } }

/// Return the `Curve` value indicating that the BN254 construction should be used in a given function.
public fun bn254(): Curve { Curve { id: 1 } }

/// A `PreparedVerifyingKey` consisting of four components in serialized form.
public struct PreparedVerifyingKey has store, copy, drop {
    vk_gamma_abc_g1_bytes: vector<u8>,
    alpha_g1_beta_g2_bytes: vector<u8>,
    gamma_g2_neg_pc_bytes: vector<u8>,
    delta_g2_neg_pc_bytes: vector<u8>,
}

/// Creates a `PreparedVerifyingKey` from bytes.
public fun pvk_from_bytes(
    vk_gamma_abc_g1_bytes: vector<u8>,
    alpha_g1_beta_g2_bytes: vector<u8>,
    gamma_g2_neg_pc_bytes: vector<u8>,
    delta_g2_neg_pc_bytes: vector<u8>,
): PreparedVerifyingKey {
    PreparedVerifyingKey {
        vk_gamma_abc_g1_bytes,
        alpha_g1_beta_g2_bytes,
        gamma_g2_neg_pc_bytes,
        delta_g2_neg_pc_bytes,
    }
}

/// Returns bytes of the four components of the `PreparedVerifyingKey`.
public fun pvk_to_bytes(pvk: PreparedVerifyingKey): vector<vector<u8>> {
    vector[
        pvk.vk_gamma_abc_g1_bytes,
        pvk.alpha_g1_beta_g2_bytes,
        pvk.gamma_g2_neg_pc_bytes,
        pvk.delta_g2_neg_pc_bytes,
    ]
}

/// A `PublicProofInputs` wrapper around its serialized bytes.
public struct PublicProofInputs has store, copy, drop {
    bytes: vector<u8>,
}

/// Creates a `PublicProofInputs` wrapper from bytes. The `bytes` parameter should be a concatenation of a number of
/// 32 bytes scalar field elements to be used as public inputs in little-endian format to a circuit.
public fun public_proof_inputs_from_bytes(bytes: vector<u8>): PublicProofInputs {
    assert!(bytes.length() % 32 == 0, EInvalidScalar);
    assert!(bytes.length() / 32 <= MaxPublicInputs, ETooManyPublicInputs);
    PublicProofInputs { bytes }
}

/// A `ProofPoints` wrapper around the serialized form of three proof points.
public struct ProofPoints has store, copy, drop {
    bytes: vector<u8>,
}

/// Creates a Groth16 `ProofPoints` from bytes.
public fun proof_points_from_bytes(bytes: vector<u8>): ProofPoints {
    ProofPoints { bytes }
}

/// @param curve: What elliptic curve construction to use. See `bls12381` and `bn254`.
/// @param verifying_key: An Arkworks canonical compressed serialization of a verifying key.
///
/// Returns four vectors of bytes representing the four components of a prepared verifying key.
/// This step computes one pairing e(P, Q), and binds the verification to one particular proof statement.
/// This can be used as inputs for the `verify_groth16_proof` function.
public fun prepare_verifying_key(curve: &Curve, verifying_key: &vector<u8>): PreparedVerifyingKey {
    prepare_verifying_key_internal(curve.id, verifying_key)
}

/// Native functions that flattens the inputs into an array and passes to the Rust native function. May abort with `EInvalidVerifyingKey` or `EInvalidCurve`.
native fun prepare_verifying_key_internal(
    curve: u8,
    verifying_key: &vector<u8>,
): PreparedVerifyingKey;

/// @param curve: What elliptic curve construction to use. See the `bls12381` and `bn254` functions.
/// @param prepared_verifying_key: Consists of four vectors of bytes representing the four components of a prepared verifying key.
/// @param public_proof_inputs: Represent inputs that are public.
/// @param proof_points: Represent three proof points.
///
/// Returns a boolean indicating whether the proof is valid.
public fun verify_groth16_proof(
    curve: &Curve,
    prepared_verifying_key: &PreparedVerifyingKey,
    public_proof_inputs: &PublicProofInputs,
    proof_points: &ProofPoints,
): bool {
    verify_groth16_proof_internal(
        curve.id,
        &prepared_verifying_key.vk_gamma_abc_g1_bytes,
        &prepared_verifying_key.alpha_g1_beta_g2_bytes,
        &prepared_verifying_key.gamma_g2_neg_pc_bytes,
        &prepared_verifying_key.delta_g2_neg_pc_bytes,
        &public_proof_inputs.bytes,
        &proof_points.bytes,
    )
}

/// Native functions that flattens the inputs into arrays of vectors and passed to the Rust native function. May abort with `EInvalidCurve` or `ETooManyPublicInputs`.
native fun verify_groth16_proof_internal(
    curve: u8,
    vk_gamma_abc_g1_bytes: &vector<u8>,
    alpha_g1_beta_g2_bytes: &vector<u8>,
    gamma_g2_neg_pc_bytes: &vector<u8>,
    delta_g2_neg_pc_bytes: &vector<u8>,
    public_proof_inputs: &vector<u8>,
    proof_points: &vector<u8>,
): bool;
