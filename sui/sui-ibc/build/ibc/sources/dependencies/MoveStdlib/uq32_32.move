// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Defines an unsigned, fixed-point numeric type with a 32-bit integer part and a 32-bit fractional
/// part. The notation `uq32_32` and `UQ32_32` is based on
/// [Q notation](https://en.wikipedia.org/wiki/Q_(number_format)). `q` indicates it a fixed-point
/// number. The `u` prefix indicates it is unsigned. The `32_32` suffix indicates the number of
/// bits, where the first number indicates the number of bits in the integer part, and the second
/// the number of bits in the fractional part--in this case 32 bits for each.
module std::uq32_32;

#[error]
const EDenominator: vector<u8> = b"Quotient specified with a zero denominator";

#[error]
const EQuotientTooSmall: vector<u8> =
    b"Quotient specified is too small, and is outside of the supported range";

#[error]
const EQuotientTooLarge: vector<u8> =
    b"Quotient specified is too large, and is outside of the supported range";

#[error]
const EOverflow: vector<u8> = b"Overflow from an arithmetic operation";

#[error]
const EDivisionByZero: vector<u8> = b"Division by zero";

/// A fixed-point numeric type with 32 integer bits and 32 fractional bits, represented by an
/// underlying 64 bit value. This is a binary representation, so decimal values may not be exactly
/// representable, but it provides more than 9 decimal digits of precision both before and after the
/// decimal point (18 digits total).
public struct UQ32_32(u64) has copy, drop, store;

/// Create a fixed-point value from a quotient specified by its numerator and denominator.
/// `from_quotient` and `from_int` should be preferred over using `from_raw`.
/// Unless the denominator is a power of two, fractions can not be represented accurately,
/// so be careful about rounding errors.
/// Aborts if the denominator is zero.
/// Aborts if the input is non-zero but so small that it will be represented as zero, e.g. smaller
/// than 2^{-32}.
/// Aborts if the input is too large, e.g. larger than or equal to 2^32.
public fun from_quotient(numerator: u64, denominator: u64): UQ32_32 {
    assert!(denominator != 0, EDenominator);

    // Scale the numerator to have 64 fractional bits and the denominator to have 32 fractional
    // bits, so that the quotient will have 32 fractional bits.
    let scaled_numerator = numerator as u128 << 64;
    let scaled_denominator = denominator as u128 << 32;
    let quotient = scaled_numerator / scaled_denominator;

    // The quotient can only be zero if the numerator is also zero.
    assert!(quotient != 0 || numerator == 0, EQuotientTooSmall);

    // Return the quotient as a fixed-point number. We first need to check whether the cast
    // can succeed.
    assert!(quotient <= std::u64::max_value!() as u128, EQuotientTooLarge);
    UQ32_32(quotient as u64)
}

/// Create a fixed-point value from an integer.
/// `from_int` and `from_quotient` should be preferred over using `from_raw`.
public fun from_int(integer: u32): UQ32_32 {
    UQ32_32((integer as u64) << 32)
}

/// Add two fixed-point numbers, `a + b`.
/// Aborts if the sum overflows.
public fun add(a: UQ32_32, b: UQ32_32): UQ32_32 {
    let sum = a.0 as u128 + (b.0 as u128);
    assert!(sum <= std::u64::max_value!() as u128, EOverflow);
    UQ32_32(sum as u64)
}

/// Subtract two fixed-point numbers, `a - b`.
/// Aborts if `a < b`.
public fun sub(a: UQ32_32, b: UQ32_32): UQ32_32 {
    assert!(a.0 >= b.0, EOverflow);
    UQ32_32(a.0 - b.0)
}

/// Multiply two fixed-point numbers, truncating any fractional part of the product.
/// Aborts if the product overflows.
public fun mul(a: UQ32_32, b: UQ32_32): UQ32_32 {
    UQ32_32(int_mul(a.0, b))
}

/// Divide two fixed-point numbers, truncating any fractional part of the quotient.
/// Aborts if the divisor is zero.
/// Aborts if the quotient overflows.
public fun div(a: UQ32_32, b: UQ32_32): UQ32_32 {
    UQ32_32(int_div(a.0, b))
}

/// Convert a fixed-point number to an integer, truncating any fractional part.
public fun to_int(a: UQ32_32): u32 {
    (a.0 >> 32) as u32
}

/// Multiply a `u64` integer by a fixed-point number, truncating any fractional part of the product.
/// Aborts if the product overflows.
public fun int_mul(val: u64, multiplier: UQ32_32): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication with u128 types and keep the full 128 bit product
    // to avoid losing accuracy.
    let unscaled_product = val as u128 * (multiplier.0 as u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    let product = unscaled_product >> 32;
    // Check whether the value is too large.
    assert!(product <= std::u64::max_value!() as u128, EOverflow);
    product as u64
}

/// Divide a `u64` integer by a fixed-point number, truncating any fractional part of the quotient.
/// Aborts if the divisor is zero.
/// Aborts if the quotient overflows.
public fun int_div(val: u64, divisor: UQ32_32): u64 {
    // Check for division by zero.
    assert!(divisor.0 != 0, EDivisionByZero);
    // First convert to 128 bits and then shift left to
    // add 32 fractional zero bits to the dividend.
    let scaled_value = val as u128 << 32;
    let quotient = scaled_value / (divisor.0 as u128);
    // Check whether the value is too large.
    assert!(quotient <= std::u64::max_value!() as u128, EOverflow);
    quotient as u64
}

/// Less than or equal to. Returns `true` if and only if `a <= a`.
public fun le(a: UQ32_32, b: UQ32_32): bool {
    a.0 <= b.0
}

/// Less than. Returns `true` if and only if `a < b`.
public fun lt(a: UQ32_32, b: UQ32_32): bool {
    a.0 < b.0
}

/// Greater than or equal to. Returns `true` if and only if `a >= b`.
public fun ge(a: UQ32_32, b: UQ32_32): bool {
    a.0 >= b.0
}

/// Greater than. Returns `true` if and only if `a > b`.
public fun gt(a: UQ32_32, b: UQ32_32): bool {
    a.0 > b.0
}

/// Accessor for the raw u64 value. Can be paired with `from_raw` to perform less common operations
/// on the raw values directly.
public fun to_raw(a: UQ32_32): u64 {
    a.0
}

/// Accessor for the raw u64 value. Can be paired with `to_raw` to perform less common operations
/// on the raw values directly.
public fun from_raw(raw_value: u64): UQ32_32 {
    UQ32_32(raw_value)
}
