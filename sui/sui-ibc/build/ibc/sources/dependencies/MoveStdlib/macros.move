// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module holds shared implementation of macros used in `std`
module std::macros;

use std::string::String;

public macro fun num_max($x: _, $y: _): _ {
    let x = $x;
    let y = $y;
    if (x > y) x
    else y
}

public macro fun num_min($x: _, $y: _): _ {
    let x = $x;
    let y = $y;
    if (x < y) x
    else y
}

public macro fun num_diff($x: _, $y: _): _ {
    let x = $x;
    let y = $y;
    if (x > y) x - y
    else y - x
}

public macro fun num_divide_and_round_up($x: _, $y: _): _ {
    let x = $x;
    let y = $y;
    if (x % y == 0) x / y
    else x / y + 1
}

public macro fun num_pow($base: _, $exponent: u8): _ {
    let mut base = $base;
    let mut exponent = $exponent;
    let mut res = 1;
    while (exponent >= 1) {
        if (exponent % 2 == 0) {
            base = base * base;
            exponent = exponent / 2;
        } else {
            res = res * base;
            exponent = exponent - 1;
        }
    };

    res
}

public macro fun num_sqrt<$T, $U>($x: $T, $bitsize: u8): $T {
    let x = $x;
    let mut bit = (1: $U) << $bitsize;
    let mut res = (0: $U);
    let mut x = x as $U;

    while (bit != 0) {
        if (x >= res + bit) {
            x = x - (res + bit);
            res = (res >> 1) + bit;
        } else {
            res = res >> 1;
        };
        bit = bit >> 2;
    };

    res as $T
}

public macro fun num_to_string($x: _): String {
    let mut x = $x;
    if (x == 0) {
        return b"0".to_string()
    };
    let mut buffer = vector[];
    while (x != 0) {
        buffer.push_back(((48 + x % 10) as u8));
        x = x / 10;
    };
    buffer.reverse();
    buffer.to_string()
}

public macro fun range_do($start: _, $stop: _, $f: |_|) {
    let mut i = $start;
    let stop = $stop;
    while (i < stop) {
        $f(i);
        i = i + 1;
    }
}

public macro fun range_do_eq($start: _, $stop: _, $f: |_|) {
    let mut i = $start;
    let stop = $stop;
    // we check `i >= stop` inside the loop instead of `i <= stop` as `while` condition to avoid
    // incrementing `i` past the MAX integer value.
    // Because of this, we need to check if `i > stop` and return early--instead of letting the
    // loop bound handle it, like in the `range_do` macro.
    if (i > stop) return;
    loop {
        $f(i);
        if (i >= stop) break;
        i = i + 1;
    }
}

public macro fun do($stop: _, $f: |_|) {
    range_do!(0, $stop, $f)
}

public macro fun do_eq($stop: _, $f: |_|) {
    range_do_eq!(0, $stop, $f)
}

public macro fun try_as_u8($x: _): Option<u8> {
    let x = $x;
    if (x > 0xFF) option::none()
    else option::some(x as u8)
}

public macro fun try_as_u16($x: _): Option<u16> {
    let x = $x;
    if (x > 0xFFFF) option::none()
    else option::some(x as u16)
}

public macro fun try_as_u32($x: _): Option<u32> {
    let x = $x;
    if (x > 0xFFFF_FFFF) option::none()
    else option::some(x as u32)
}

public macro fun try_as_u64($x: _): Option<u64> {
    let x = $x;
    if (x > 0xFFFF_FFFF_FFFF_FFFF) option::none()
    else option::some(x as u64)
}

public macro fun try_as_u128($x: _): Option<u128> {
    let x = $x;
    if (x > 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF) option::none()
    else option::some(x as u128)
}
