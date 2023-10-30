// Copyright 2015 blake2-rfc Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

mod simd_opt;
mod simdint;
mod simdop;
mod simdty;

pub use self::simdty::{u32x4, u64x4};

pub trait Vector4<T>: Copy {
    fn gather(src: &[T], i0: usize, i1: usize, i2: usize, i3: usize) -> Self;

    #[allow(clippy::wrong_self_convention)]
    fn from_le(self) -> Self;
    fn to_le(self) -> Self;

    fn wrapping_add(self, rhs: Self) -> Self;

    fn rotate_right_const(self, n: u32) -> Self;

    fn shuffle_left_1(self) -> Self;
    fn shuffle_left_2(self) -> Self;
    fn shuffle_left_3(self) -> Self;

    #[inline(always)]
    fn shuffle_right_1(self) -> Self {
        self.shuffle_left_3()
    }
    #[inline(always)]
    fn shuffle_right_2(self) -> Self {
        self.shuffle_left_2()
    }
    #[inline(always)]
    fn shuffle_right_3(self) -> Self {
        self.shuffle_left_1()
    }
}

macro_rules! impl_vector4 {
    ($vec:ident, $word:ident) => {
        impl Vector4<$word> for $vec {
            #[inline(always)]
            fn gather(src: &[$word], i0: usize, i1: usize, i2: usize, i3: usize) -> Self {
                $vec::new(src[i0], src[i1], src[i2], src[i3])
            }

            #[cfg(target_endian = "little")]
            #[inline(always)]
            fn from_le(self) -> Self {
                self
            }

            #[cfg(not(target_endian = "little"))]
            #[inline(always)]
            fn from_le(self) -> Self {
                $vec::new(
                    $word::from_le(self.0),
                    $word::from_le(self.1),
                    $word::from_le(self.2),
                    $word::from_le(self.3),
                )
            }

            #[cfg(target_endian = "little")]
            #[inline(always)]
            fn to_le(self) -> Self {
                self
            }

            #[cfg(not(target_endian = "little"))]
            #[inline(always)]
            fn to_le(self) -> Self {
                $vec::new(
                    self.0.to_le(),
                    self.1.to_le(),
                    self.2.to_le(),
                    self.3.to_le(),
                )
            }

            #[inline(always)]
            fn wrapping_add(self, rhs: Self) -> Self {
                self + rhs
            }

            #[inline(always)]
            fn rotate_right_const(self, n: u32) -> Self {
                simd_opt::$vec::rotate_right_const(self, n)
            }

            #[cfg(feature = "simd")]
            #[inline(always)]
            fn shuffle_left_1(self) -> Self {
                use crate::simd::simdint::simd_shuffle4;
                const IDX: [u32; 4] = [1, 2, 3, 0];
                unsafe { simd_shuffle4(self, self, IDX) }
            }

            #[cfg(not(feature = "simd"))]
            #[inline(always)]
            fn shuffle_left_1(self) -> Self {
                $vec::new(self.1, self.2, self.3, self.0)
            }

            #[cfg(feature = "simd")]
            #[inline(always)]
            fn shuffle_left_2(self) -> Self {
                use crate::simd::simdint::simd_shuffle4;
                const IDX: [u32; 4] = [2, 3, 0, 1];
                unsafe { simd_shuffle4(self, self, IDX) }
            }

            #[cfg(not(feature = "simd"))]
            #[inline(always)]
            fn shuffle_left_2(self) -> Self {
                $vec::new(self.2, self.3, self.0, self.1)
            }

            #[cfg(feature = "simd")]
            #[inline(always)]
            fn shuffle_left_3(self) -> Self {
                use crate::simd::simdint::simd_shuffle4;
                const IDX: [u32; 4] = [3, 0, 1, 2];
                unsafe { simd_shuffle4(self, self, IDX) }
            }

            #[cfg(not(feature = "simd"))]
            #[inline(always)]
            fn shuffle_left_3(self) -> Self {
                $vec::new(self.3, self.0, self.1, self.2)
            }
        }
    };
}

impl_vector4!(u32x4, u32);
impl_vector4!(u64x4, u64);
