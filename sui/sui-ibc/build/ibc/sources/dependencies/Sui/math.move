// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// DEPRECATED, use the each integer type's individual module instead, e.g. `std::u64`
#[deprecated(note = b"Use the each integer type's individual module instead, e.g. `std::u64`")]
module sui::math;

/// DEPRECATED, use `std::u64::max` instead
public fun max(x: u64, y: u64): u64 {
    x.max(y)
}

/// DEPRECATED, use `std::u64::min` instead
public fun min(x: u64, y: u64): u64 {
    x.min(y)
}

/// DEPRECATED, use `std::u64::diff` instead
public fun diff(x: u64, y: u64): u64 {
    x.diff(y)
}

/// DEPRECATED, use `std::u64::pow` instead
public fun pow(base: u64, exponent: u8): u64 {
    base.pow(exponent)
}

/// DEPRECATED, use `std::u64::sqrt` instead
public fun sqrt(x: u64): u64 {
    x.sqrt()
}

/// DEPRECATED, use `std::u128::sqrt` instead
public fun sqrt_u128(x: u128): u128 {
    x.sqrt()
}

/// DEPRECATED, use `std::u64::divide_and_round_up` instead
public fun divide_and_round_up(x: u64, y: u64): u64 {
    x.divide_and_round_up(y)
}
