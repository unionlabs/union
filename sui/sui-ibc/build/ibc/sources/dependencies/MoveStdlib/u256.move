// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[defines_primitive(u256)]
module std::u256;

use std::string::String;

/// Returns the bitwise not of the value.
/// Each bit that is 1 becomes 0. Each bit that is 0 becomes 1.
public fun bitwise_not(x: u256): u256 {
    x ^ max_value!()
}

/// Return the larger of `x` and `y`
public fun max(x: u256, y: u256): u256 {
    std::macros::num_max!(x, y)
}

/// Return the smaller of `x` and `y`
public fun min(x: u256, y: u256): u256 {
    std::macros::num_min!(x, y)
}

/// Return the absolute value of x - y
public fun diff(x: u256, y: u256): u256 {
    std::macros::num_diff!(x, y)
}

/// Calculate x / y, but round up the result.
public fun divide_and_round_up(x: u256, y: u256): u256 {
    std::macros::num_divide_and_round_up!(x, y)
}

/// Return the value of a base raised to a power
public fun pow(base: u256, exponent: u8): u256 {
    std::macros::num_pow!(base, exponent)
}

/// Try to convert a `u256` to a `u8`. Returns `None` if the value is too large.
public fun try_as_u8(x: u256): Option<u8> {
    std::macros::try_as_u8!(x)
}

/// Try to convert a `u256` to a `u16`. Returns `None` if the value is too large.
public fun try_as_u16(x: u256): Option<u16> {
    std::macros::try_as_u16!(x)
}

/// Try to convert a `u256` to a `u32`. Returns `None` if the value is too large.
public fun try_as_u32(x: u256): Option<u32> {
    std::macros::try_as_u32!(x)
}

/// Try to convert a `u256` to a `u64`. Returns `None` if the value is too large.
public fun try_as_u64(x: u256): Option<u64> {
    std::macros::try_as_u64!(x)
}

/// Try to convert a `u256` to a `u128`. Returns `None` if the value is too large.
public fun try_as_u128(x: u256): Option<u128> {
    std::macros::try_as_u128!(x)
}

public fun to_string(x: u256): String {
    std::macros::num_to_string!(x)
}

/// Maximum value for a `u256`
public macro fun max_value(): u256 {
    0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF
}

/// Loops applying `$f` to each number from `$start` to `$stop` (exclusive)
public macro fun range_do($start: u256, $stop: u256, $f: |u256|) {
    std::macros::range_do!($start, $stop, $f)
}

/// Loops applying `$f` to each number from `$start` to `$stop` (inclusive)
public macro fun range_do_eq($start: u256, $stop: u256, $f: |u256|) {
    std::macros::range_do_eq!($start, $stop, $f)
}

/// Loops applying `$f` to each number from `0` to `$stop` (exclusive)
public macro fun do($stop: u256, $f: |u256|) {
    std::macros::do!($stop, $f)
}

/// Loops applying `$f` to each number from `0` to `$stop` (inclusive)
public macro fun do_eq($stop: u256, $f: |u256|) {
    std::macros::do_eq!($stop, $f)
}
