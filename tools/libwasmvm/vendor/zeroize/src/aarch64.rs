//! [`Zeroize`] impls for ARM64 SIMD registers.
//!
//! Gated behind the `aarch64` feature: MSRV 1.59
//! (the overall crate is MSRV 1.51)

use crate::{atomic_fence, volatile_write, Zeroize};

use core::arch::aarch64::*;

macro_rules! impl_zeroize_for_simd_register {
    ($(($type:ty, $vdupq:ident)),+) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(target_arch = "aarch64")))]
            #[cfg_attr(docsrs, doc(cfg(target_feature = "neon")))]
            impl Zeroize for $type {
                fn zeroize(&mut self) {
                    volatile_write(self, unsafe { $vdupq(0) });
                    atomic_fence();
                }
            }
        )+
    };
}

// TODO(tarcieri): other NEON register types?
impl_zeroize_for_simd_register! {
    (uint8x8_t, vdup_n_u8),
    (uint8x16_t, vdupq_n_u8),
    (uint16x4_t, vdup_n_u16),
    (uint16x8_t, vdupq_n_u16),
    (uint32x2_t, vdup_n_u32),
    (uint32x4_t, vdupq_n_u32),
    (uint64x1_t, vdup_n_u64),
    (uint64x2_t, vdupq_n_u64)
}
