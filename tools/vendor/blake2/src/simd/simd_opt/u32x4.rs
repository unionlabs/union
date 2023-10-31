// Copyright 2015 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::simd::simdty::u32x4;

#[cfg(feature = "simd_opt")]
#[inline(always)]
pub fn rotate_right_const(vec: u32x4, n: u32) -> u32x4 {
    match n {
        16 => rotate_right_16(vec),
        8 => rotate_right_8(vec),
        _ => rotate_right_any(vec, n),
    }
}

#[cfg(not(feature = "simd_opt"))]
#[inline(always)]
pub fn rotate_right_const(vec: u32x4, n: u32) -> u32x4 {
    rotate_right_any(vec, n)
}

#[inline(always)]
fn rotate_right_any(vec: u32x4, n: u32) -> u32x4 {
    let r = n as u32;
    let l = 32 - r;

    (vec >> u32x4::new(r, r, r, r)) ^ (vec << u32x4::new(l, l, l, l))
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_16(vec: u32x4) -> u32x4 {
    if cfg!(target_feature = "ssse3") {
        // pshufb (SSSE3) / vpshufb (AVX2)
        transmute_shuffle!(
            u8x16,
            simd_shuffle16,
            vec,
            16,
            [2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13]
        )
    } else if cfg!(any(target_feature = "sse2", target_feature = "neon")) {
        // pshuflw+pshufhw (SSE2) / vrev (NEON)
        transmute_shuffle!(u16x8, simd_shuffle8, vec, 8, [1, 0, 3, 2, 5, 4, 7, 6])
    } else {
        rotate_right_any(vec, 16)
    }
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_8(vec: u32x4) -> u32x4 {
    if cfg!(target_feature = "ssse3") {
        // pshufb (SSSE3) / vpshufb (AVX2)
        transmute_shuffle!(
            u8x16,
            simd_shuffle16,
            vec,
            16,
            [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12]
        )
    } else {
        rotate_right_any(vec, 8)
    }
}
