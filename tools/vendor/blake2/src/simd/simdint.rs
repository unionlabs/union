// Copyright 2015 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

#[cfg(feature = "simd")]
extern "platform-intrinsic" {
    pub fn simd_add<T>(x: T, y: T) -> T;
    pub fn simd_shl<T>(x: T, y: T) -> T;
    pub fn simd_shr<T>(x: T, y: T) -> T;
    pub fn simd_xor<T>(x: T, y: T) -> T;

    pub fn simd_shuffle2<T, U>(v: T, w: T, idx: [u32; 2]) -> U;
    pub fn simd_shuffle4<T, U>(v: T, w: T, idx: [u32; 4]) -> U;
    pub fn simd_shuffle8<T, U>(v: T, w: T, idx: [u32; 8]) -> U;
    pub fn simd_shuffle16<T, U>(v: T, w: T, idx: [u32; 16]) -> U;
    pub fn simd_shuffle32<T, U>(v: T, w: T, idx: [u32; 32]) -> U;
}
