// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module implements BCS (de)serialization in Move.
/// Full specification can be found here: https://github.com/diem/bcs
///
/// Short summary (for Move-supported types):
///
/// - address - sequence of X bytes
/// - bool - byte with 0 or 1
/// - u8 - a single u8 byte
/// - u16 / u32 / u64 / u128 / u256 - LE bytes
/// - vector - ULEB128 length + LEN elements
/// - option - first byte bool: None (0) or Some (1), then value
///
/// Usage example:
/// ```
/// /// This function reads u8 and u64 value from the input
/// /// and returns the rest of the bytes.
/// fun deserialize(bytes: vector<u8>): (u8, u64, vector<u8>) {
///     use sui::bcs::{Self, BCS};
///
///     let prepared: BCS = bcs::new(bytes);
///     let (u8_value, u64_value) = (
///         prepared.peel_u8(),
///         prepared.peel_u64()
///     );
///
///     // unpack bcs struct
///     let leftovers = prepared.into_remainder_bytes();
///
///     (u8_value, u64_value, leftovers)
/// }
/// ```
module sui::bcs;

use std::bcs;
use sui::address;

/// For when bytes length is less than required for deserialization.
const EOutOfRange: u64 = 0;
/// For when the boolean value different than `0` or `1`.
const ENotBool: u64 = 1;
/// For when ULEB byte is out of range (or not found).
const ELenOutOfRange: u64 = 2;

/// A helper struct that saves resources on operations. For better
/// vector performance, it stores reversed bytes of the BCS and
/// enables use of `vector::pop_back`.
public struct BCS has store, copy, drop {
    bytes: vector<u8>,
}

/// Get BCS serialized bytes for any value.
/// Re-exports stdlib `bcs::to_bytes`.
public fun to_bytes<T>(value: &T): vector<u8> {
    bcs::to_bytes(value)
}

/// Creates a new instance of BCS wrapper that holds inversed
/// bytes for better performance.
public fun new(mut bytes: vector<u8>): BCS {
    bytes.reverse();
    BCS { bytes }
}

/// Unpack the `BCS` struct returning the leftover bytes.
/// Useful for passing the data further after partial deserialization.
public fun into_remainder_bytes(bcs: BCS): vector<u8> {
    let BCS { mut bytes } = bcs;
    bytes.reverse();
    bytes
}

/// Read address from the bcs-serialized bytes.
public fun peel_address(bcs: &mut BCS): address {
    assert!(bcs.bytes.length() >= address::length(), EOutOfRange);
    let (mut addr_bytes, mut i) = (vector[], 0);
    while (i < address::length()) {
        addr_bytes.push_back(bcs.bytes.pop_back());
        i = i + 1;
    };
    address::from_bytes(addr_bytes)
}

/// Read a `bool` value from bcs-serialized bytes.
public fun peel_bool(bcs: &mut BCS): bool {
    let value = bcs.peel_u8();
    if (value == 0) false
    else if (value == 1) true
    else abort ENotBool
}

/// Read `u8` value from bcs-serialized bytes.
public fun peel_u8(bcs: &mut BCS): u8 {
    assert!(bcs.bytes.length() >= 1, EOutOfRange);
    bcs.bytes.pop_back()
}

macro fun peel_num<$I, $T>($bcs: &mut BCS, $len: u64, $bits: $I): $T {
    let bcs = $bcs;
    assert!(bcs.bytes.length() >= $len, EOutOfRange);

    let mut value: $T = 0;
    let mut i: $I = 0;
    let bits = $bits;
    while (i < bits) {
        let byte = bcs.bytes.pop_back() as $T;
        value = value + (byte << (i as u8));
        i = i + 8;
    };

    value
}

/// Read `u16` value from bcs-serialized bytes.
public fun peel_u16(bcs: &mut BCS): u16 {
    bcs.peel_num!(2, 16u8)
}

/// Read `u32` value from bcs-serialized bytes.
public fun peel_u32(bcs: &mut BCS): u32 {
    bcs.peel_num!(4, 32u8)
}

/// Read `u64` value from bcs-serialized bytes.
public fun peel_u64(bcs: &mut BCS): u64 {
    bcs.peel_num!(8, 64u8)
}

/// Read `u128` value from bcs-serialized bytes.
public fun peel_u128(bcs: &mut BCS): u128 {
    bcs.peel_num!(16, 128u8)
}

/// Read `u256` value from bcs-serialized bytes.
public fun peel_u256(bcs: &mut BCS): u256 {
    bcs.peel_num!(32, 256u16)
}

// === Vector<T> ===

/// Read ULEB bytes expecting a vector length. Result should
/// then be used to perform `peel_*` operation LEN times.
///
/// In BCS `vector` length is implemented with ULEB128;
/// See more here: https://en.wikipedia.org/wiki/LEB128
public fun peel_vec_length(bcs: &mut BCS): u64 {
    let (mut total, mut shift, mut len) = (0u64, 0, 0);
    loop {
        assert!(len <= 4, ELenOutOfRange);
        let byte = bcs.bytes.pop_back() as u64;
        len = len + 1;
        total = total | ((byte & 0x7f) << shift);
        if ((byte & 0x80) == 0) break;
        shift = shift + 7;
    };
    total
}

/// Peel `vector<$T>` from serialized bytes, where `$peel: |&mut BCS| -> $T` gives the
/// functionality of peeling each value.
public macro fun peel_vec<$T>($bcs: &mut BCS, $peel: |&mut BCS| -> $T): vector<$T> {
    let bcs = $bcs;
    let len = bcs.peel_vec_length();
    let mut i = 0;
    let mut res = vector[];
    while (i < len) {
        res.push_back($peel(bcs));
        i = i + 1;
    };
    res
}

/// Peel a vector of `address` from serialized bytes.
public fun peel_vec_address(bcs: &mut BCS): vector<address> {
    bcs.peel_vec!(|bcs| bcs.peel_address())
}

/// Peel a vector of `address` from serialized bytes.
public fun peel_vec_bool(bcs: &mut BCS): vector<bool> {
    bcs.peel_vec!(|bcs| bcs.peel_bool())
}

/// Peel a vector of `u8` (eg string) from serialized bytes.
public fun peel_vec_u8(bcs: &mut BCS): vector<u8> {
    bcs.peel_vec!(|bcs| bcs.peel_u8())
}

/// Peel a `vector<vector<u8>>` (eg vec of string) from serialized bytes.
public fun peel_vec_vec_u8(bcs: &mut BCS): vector<vector<u8>> {
    bcs.peel_vec!(|bcs| bcs.peel_vec_u8())
}

/// Peel a vector of `u16` from serialized bytes.
public fun peel_vec_u16(bcs: &mut BCS): vector<u16> {
    bcs.peel_vec!(|bcs| bcs.peel_u16())
}

/// Peel a vector of `u32` from serialized bytes.
public fun peel_vec_u32(bcs: &mut BCS): vector<u32> {
    bcs.peel_vec!(|bcs| bcs.peel_u32())
}

/// Peel a vector of `u64` from serialized bytes.
public fun peel_vec_u64(bcs: &mut BCS): vector<u64> {
    bcs.peel_vec!(|bcs| bcs.peel_u64())
}

/// Peel a vector of `u128` from serialized bytes.
public fun peel_vec_u128(bcs: &mut BCS): vector<u128> {
    bcs.peel_vec!(|bcs| bcs.peel_u128())
}

/// Peel a vector of `u256` from serialized bytes.
public fun peel_vec_u256(bcs: &mut BCS): vector<u256> {
    bcs.peel_vec!(|bcs| bcs.peel_u256())
}

// === Option<T> ===

/// Peel `Option<$T>` from serialized bytes, where `$peel: |&mut BCS| -> $T` gives the
/// functionality of peeling the inner value.
public macro fun peel_option<$T>($bcs: &mut BCS, $peel: |&mut BCS| -> $T): Option<$T> {
    let bcs = $bcs;
    if (bcs.peel_bool()) option::some($peel(bcs))
    else option::none()
}

/// Peel `Option<address>` from serialized bytes.
public fun peel_option_address(bcs: &mut BCS): Option<address> {
    bcs.peel_option!(|bcs| bcs.peel_address())
}

/// Peel `Option<bool>` from serialized bytes.
public fun peel_option_bool(bcs: &mut BCS): Option<bool> {
    bcs.peel_option!(|bcs| bcs.peel_bool())
}

/// Peel `Option<u8>` from serialized bytes.
public fun peel_option_u8(bcs: &mut BCS): Option<u8> {
    bcs.peel_option!(|bcs| bcs.peel_u8())
}

/// Peel `Option<u16>` from serialized bytes.
public fun peel_option_u16(bcs: &mut BCS): Option<u16> {
    bcs.peel_option!(|bcs| bcs.peel_u16())
}

/// Peel `Option<u32>` from serialized bytes.
public fun peel_option_u32(bcs: &mut BCS): Option<u32> {
    bcs.peel_option!(|bcs| bcs.peel_u32())
}

/// Peel `Option<u64>` from serialized bytes.
public fun peel_option_u64(bcs: &mut BCS): Option<u64> {
    bcs.peel_option!(|bcs| bcs.peel_u64())
}

/// Peel `Option<u128>` from serialized bytes.
public fun peel_option_u128(bcs: &mut BCS): Option<u128> {
    bcs.peel_option!(|bcs| bcs.peel_u128())
}

/// Peel `Option<u256>` from serialized bytes.
public fun peel_option_u256(bcs: &mut BCS): Option<u256> {
    bcs.peel_option!(|bcs| bcs.peel_u256())
}
