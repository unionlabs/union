// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Module which defines SHA hashes for byte vectors.
///
/// The functions in this module are natively declared both in the Move runtime
/// as in the Move prover's prelude.
module std::hash;

public native fun sha2_256(data: vector<u8>): vector<u8>;
public native fun sha3_256(data: vector<u8>): vector<u8>;
