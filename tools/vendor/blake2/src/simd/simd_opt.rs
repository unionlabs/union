// Copyright 2015 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[allow(unused_macros)]
#[cfg(feature = "simd")]
macro_rules! transmute_shuffle {
    ($tmp:ident, $shuffle:ident, $vec:expr, $idx_n:expr, $idx:expr) => {
        unsafe {
            use crate::simd::simdint::$shuffle;
            use crate::simd::simdty::$tmp;
            use core::mem::transmute;

            const IDX: [u32; $idx_n] = $idx;
            let tmp_i: $tmp = transmute($vec);
            let tmp_o: $tmp = $shuffle(tmp_i, tmp_i, IDX);
            transmute(tmp_o)
        }
    };
}

#[cfg(feature = "simd")]
pub mod u32x4;
#[cfg(feature = "simd")]
pub mod u64x4;

#[cfg(not(feature = "simd"))]
macro_rules! simd_opt {
    ($vec:ident) => {
        pub mod $vec {
            use crate::simd::simdty::$vec;

            #[inline(always)]
            pub fn rotate_right_const(vec: $vec, n: u32) -> $vec {
                $vec::new(
                    vec.0.rotate_right(n),
                    vec.1.rotate_right(n),
                    vec.2.rotate_right(n),
                    vec.3.rotate_right(n),
                )
            }
        }
    };
}

#[cfg(not(feature = "simd"))]
simd_opt!(u32x4);
#[cfg(not(feature = "simd"))]
simd_opt!(u64x4);
