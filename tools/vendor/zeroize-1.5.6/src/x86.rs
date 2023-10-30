//! [`Zeroize`] impls for x86 SIMD registers

use crate::{atomic_fence, volatile_write, Zeroize};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

macro_rules! impl_zeroize_for_simd_register {
    ($type:ty, $feature:expr, $zero_value:ident) => {
        #[cfg_attr(docsrs, doc(cfg(target_arch = "x86")))] // also `x86_64`
        #[cfg_attr(docsrs, doc(cfg(target_feature = $feature)))]
        impl Zeroize for $type {
            fn zeroize(&mut self) {
                volatile_write(self, unsafe { $zero_value() });
                atomic_fence();
            }
        }
    };
}

#[cfg(target_feature = "sse")]
impl_zeroize_for_simd_register!(__m128, "sse", _mm_setzero_ps);

#[cfg(target_feature = "sse2")]
impl_zeroize_for_simd_register!(__m128d, "sse2", _mm_setzero_pd);

#[cfg(target_feature = "sse2")]
impl_zeroize_for_simd_register!(__m128i, "sse2", _mm_setzero_si128);

#[cfg(target_feature = "avx")]
impl_zeroize_for_simd_register!(__m256, "avx", _mm256_setzero_ps);

#[cfg(target_feature = "avx")]
impl_zeroize_for_simd_register!(__m256d, "avx", _mm256_setzero_pd);

#[cfg(target_feature = "avx")]
impl_zeroize_for_simd_register!(__m256i, "avx", _mm256_setzero_si256);
