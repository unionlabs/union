// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Module which defines instances of the poseidon hash functions.
module sui::poseidon;

use sui::bcs;

/// Error if any of the inputs are larger than or equal to the BN254 field size.
const ENonCanonicalInput: u64 = 0;

/// Error if an empty vector is passed as input.
const EEmptyInput: u64 = 1;

/// The field size for BN254 curve.
const BN254_MAX: u256 =
    21888242871839275222246405745257275088548364400416034343698204186575808495617u256;

/// @param data: Vector of BN254 field elements to hash.
///
/// Hash the inputs using poseidon_bn254 and returns a BN254 field element.
///
/// Each element has to be a BN254 field element in canonical representation so it must be smaller than the BN254
/// scalar field size which is 21888242871839275222246405745257275088548364400416034343698204186575808495617.
public fun poseidon_bn254(data: &vector<u256>): u256 {
    let (mut i, mut b, l) = (0, vector[], data.length());
    assert!(l > 0, EEmptyInput);
    while (i < l) {
        let field_element = &data[i];
        assert!(*field_element < BN254_MAX, ENonCanonicalInput);
        b.push_back(bcs::to_bytes(&data[i]));
        i = i + 1;
    };
    let binary_output = poseidon_bn254_internal(&b);
    bcs::new(binary_output).peel_u256()
}

/// @param data: Vector of BN254 field elements in little-endian representation.
///
/// Hash the inputs using poseidon_bn254 and returns a BN254 field element in little-endian representation.
native fun poseidon_bn254_internal(data: &vector<vector<u8>>): vector<u8>;
