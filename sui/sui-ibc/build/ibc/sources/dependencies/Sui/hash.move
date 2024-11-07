// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Module which defines hash functions. Note that Sha-256 and Sha3-256 is available in the std::hash module in the
/// standard library.
module sui::hash;

/// @param data: Arbitrary binary data to hash
/// Hash the input bytes using Blake2b-256 and returns 32 bytes.
public native fun blake2b256(data: &vector<u8>): vector<u8>;

/// @param data: Arbitrary binary data to hash
/// Hash the input bytes using keccak256 and returns 32 bytes.
public native fun keccak256(data: &vector<u8>): vector<u8>;
