// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Utility for converting a Move value to its binary representation in BCS (Binary Canonical
/// Serialization). BCS is the binary encoding for Move resources and other non-module values
/// published on-chain. See https://github.com/diem/bcs#binary-canonical-serialization-bcs for more
/// details on BCS.
module std::bcs;

/// Return the binary representation of `v` in BCS (Binary Canonical Serialization) format
public native fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;
