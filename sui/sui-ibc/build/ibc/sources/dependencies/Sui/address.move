// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[defines_primitive(address)]
module sui::address;

use std::ascii;
use std::bcs;
use std::string;
use sui::hex;

/// Allows calling `.to_id()` on an address to get its `ID`.
public use fun sui::object::id_from_address as address.to_id;

/// The length of an address, in bytes
const LENGTH: u64 = 32;

// The largest integer that can be represented with 32 bytes: 2^(8*32) - 1
const MAX: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

#[allow(unused_const)]
/// Error from `from_bytes` when it is supplied too many or too few bytes.
const EAddressParseError: u64 = 0;

/// Convert `a` into a u256 by interpreting `a` as the bytes of a big-endian integer
/// (e.g., `to_u256(0x1) == 1`)
public native fun to_u256(a: address): u256;

/// Convert `n` into an address by encoding it as a big-endian integer (e.g., `from_u256(1) = @0x1`)
/// Aborts if `n` > `MAX_ADDRESS`
public native fun from_u256(n: u256): address;

/// Convert `bytes` into an address.
/// Aborts with `EAddressParseError` if the length of `bytes` is not 32
public native fun from_bytes(bytes: vector<u8>): address;

/// Convert `a` into BCS-encoded bytes.
public fun to_bytes(a: address): vector<u8> {
    bcs::to_bytes(&a)
}

/// Convert `a` to a hex-encoded ASCII string
public fun to_ascii_string(a: address): ascii::String {
    hex::encode(to_bytes(a)).to_ascii_string()
}

/// Convert `a` to a hex-encoded string
public fun to_string(a: address): string::String {
    to_ascii_string(a).to_string()
}

/// Converts an ASCII string to an address, taking the numerical value for each character. The
/// string must be Base16 encoded, and thus exactly 64 characters long.
/// For example, the string "00000000000000000000000000000000000000000000000000000000DEADB33F"
/// will be converted to the address @0xDEADB33F.
/// Aborts with `EAddressParseError` if the length of `s` is not 64,
/// or if an invalid character is encountered.
public fun from_ascii_bytes(bytes: &vector<u8>): address {
    assert!(bytes.length() == 64, EAddressParseError);
    let mut hex_bytes = vector[];
    let mut i = 0;
    while (i < 64) {
        let hi = hex_char_value(bytes[i]);
        let lo = hex_char_value(bytes[i+1]);
        hex_bytes.push_back((hi << 4) | lo);
        i = i + 2;
    };
    from_bytes(hex_bytes)
}

fun hex_char_value(c: u8): u8 {
    if (c >= 48 && c <= 57) c - 48 // 0-9
    else if (c >= 65 && c <= 70) c - 55 // A-F
    else if (c >= 97 && c <= 102) c - 87 // a-f
    else abort EAddressParseError
}

/// Length of a Sui address in bytes
public fun length(): u64 {
    LENGTH
}

/// Largest possible address
public fun max(): u256 {
    MAX
}
