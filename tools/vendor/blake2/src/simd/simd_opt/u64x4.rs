// Copyright 2015 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::simd::simdty::u64x4;

#[cfg(feature = "simd_opt")]
#[inline(always)]
pub fn rotate_right_const(vec: u64x4, n: u32) -> u64x4 {
    match n {
        32 => rotate_right_32(vec),
        24 => rotate_right_24(vec),
        16 => rotate_right_16(vec),
        _ => rotate_right_any(vec, n),
    }
}

#[cfg(not(feature = "simd_opt"))]
#[inline(always)]
pub fn rotate_right_const(vec: u64x4, n: u32) -> u64x4 {
    rotate_right_any(vec, n)
}

#[inline(always)]
fn rotate_right_any(vec: u64x4, n: u32) -> u64x4 {
    let r = n as u64;
    let l = 64 - r;

    (vec >> u64x4::new(r, r, r, r)) ^ (vec << u64x4::new(l, l, l, l))
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_32(vec: u64x4) -> u64x4 {
    if cfg!(any(target_feature = "sse2", target_feature = "neon")) {
        // 2 x pshufd (SSE2) / vpshufd (AVX2) / 2 x vrev (NEON)
        transmute_shuffle!(u32x8, simd_shuffle8, vec, 8, [1, 0, 3, 2, 5, 4, 7, 6])
    } else {
        rotate_right_any(vec, 32)
    }
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_24(vec: u64x4) -> u64x4 {
    if cfg!(all(
        feature = "simd_asm",
        target_feature = "neon",
        target_arch = "arm"
    )) {
        // 4 x vext (NEON)
        rotate_right_vext(vec, 3)
    } else if cfg!(target_feature = "ssse3") {
        // 2 x pshufb (SSSE3) / vpshufb (AVX2)
        transmute_shuffle!(
            u8x32,
            simd_shuffle32,
            vec,
            32,
            [
                3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13, 14, 15, 8, 9, 10, 19, 20, 21, 22, 23, 16, 17,
                18, 27, 28, 29, 30, 31, 24, 25, 26
            ]
        )
    } else {
        rotate_right_any(vec, 24)
    }
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_16(vec: u64x4) -> u64x4 {
    if cfg!(all(
        feature = "simd_asm",
        target_feature = "neon",
        target_arch = "arm"
    )) {
        // 4 x vext (NEON)
        rotate_right_vext(vec, 2)
    } else if cfg!(target_feature = "ssse3") {
        // 2 x pshufb (SSSE3) / vpshufb (AVX2)
        transmute_shuffle!(
            u8x32,
            simd_shuffle32,
            vec,
            32,
            [
                2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9, 18, 19, 20, 21, 22, 23, 16,
                17, 26, 27, 28, 29, 30, 31, 24, 25
            ]
        )
    } else if cfg!(target_feature = "sse2") {
        // 2 x pshuflw+pshufhw (SSE2)
        transmute_shuffle!(
            u16x16,
            simd_shuffle16,
            vec,
            16,
            [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12]
        )
    } else {
        rotate_right_any(vec, 16)
    }
}

#[cfg(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm"))]
mod simd_asm_neon_arm {
    use crate::simd::simdty::{u64x2, u64x4};

    #[inline(always)]
    fn vext_u64(vec: u64x2, b: u8) -> u64x2 {
        unsafe {
            let result: u64x2;
            asm!("vext.8 ${0:e}, ${1:e}, ${1:e}, $2\nvext.8 ${0:f}, ${1:f}, ${1:f}, $2"
               : "=w" (result)
               : "w" (vec), "n" (b));
            result
        }
    }

    #[inline(always)]
    pub fn rotate_right_vext(vec: u64x4, b: u8) -> u64x4 {
        use crate::simd::simdint::{simd_shuffle2, simd_shuffle4};

        unsafe {
            let tmp0 = vext_u64(simd_shuffle2(vec, vec, [0, 1]), b);
            let tmp1 = vext_u64(simd_shuffle2(vec, vec, [2, 3]), b);
            simd_shuffle4(tmp0, tmp1, [0, 1, 2, 3])
        }
    }
}

#[cfg(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm"))]
use self::simd_asm_neon_arm::rotate_right_vext;

#[cfg(feature = "simd_opt")]
#[cfg(not(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm")))]
fn rotate_right_vext(_vec: u64x4, _n: u8) -> u64x4 {
    unreachable!()
}
