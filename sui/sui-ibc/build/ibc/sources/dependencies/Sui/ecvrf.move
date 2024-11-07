// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::ecvrf;

#[allow(unused_const)]
const EInvalidHashLength: u64 = 1;
#[allow(unused_const)]
const EInvalidPublicKeyEncoding: u64 = 2;
#[allow(unused_const)]
const EInvalidProofEncoding: u64 = 3;

/// @param hash: The hash/output from a ECVRF to be verified.
/// @param alpha_string: Input/seed to the ECVRF used to generate the output.
/// @param public_key: The public key corresponding to the private key used to generate the output.
/// @param proof: The proof of validity of the output.
/// Verify a proof for a Ristretto ECVRF. Returns true if the proof is valid and corresponds to the given output. May abort with `EInvalidHashLength`, `EInvalidPublicKeyEncoding` or `EInvalidProofEncoding`.
public native fun ecvrf_verify(
    hash: &vector<u8>,
    alpha_string: &vector<u8>,
    public_key: &vector<u8>,
    proof: &vector<u8>,
): bool;
