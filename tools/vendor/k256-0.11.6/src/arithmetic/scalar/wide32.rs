//! Wide scalar (32-bit limbs)

use super::{Scalar, MODULUS};
use crate::ORDER;
use elliptic_curve::{
    bigint::{Limb, U256, U512},
    subtle::{Choice, ConditionallySelectable},
};

/// Limbs of 2^256 minus the secp256k1 order.
const NEG_MODULUS: [u32; 8] = [
    !MODULUS[0] + 1,
    !MODULUS[1],
    !MODULUS[2],
    !MODULUS[3],
    !MODULUS[4],
    !MODULUS[5],
    !MODULUS[6],
    !MODULUS[7],
];

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WideScalar(pub(super) U512);

impl WideScalar {
    pub const fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self(U512::from_be_slice(bytes))
    }

    /// Multiplies two scalars without modulo reduction, producing up to a 512-bit scalar.
    #[inline(always)] // only used in Scalar::mul(), so won't cause binary bloat
    pub fn mul_wide(a: &Scalar, b: &Scalar) -> Self {
        let a = a.0.to_words();
        let b = b.0.to_words();

        // 96 bit accumulator.
        let c0 = 0;
        let c1 = 0;
        let c2 = 0;

        // l[0..15] = a[0..7] * b[0..7].
        let (c0, c1) = muladd_fast(a[0], b[0], c0, c1);
        let (l0, c0, c1) = (c0, c1, 0);
        let (c0, c1, c2) = muladd(a[0], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[0], c0, c1, c2);
        let (l1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[0], c0, c1, c2);
        let (l2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[0], c0, c1, c2);
        let (l3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[0], c0, c1, c2);
        let (l4, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[0], c0, c1, c2);
        let (l5, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[0], c0, c1, c2);
        let (l6, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[0], c0, c1, c2);
        let (l7, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[1], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[1], c0, c1, c2);
        let (l8, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[2], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[2], c0, c1, c2);
        let (l9, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[3], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[4], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[3], c0, c1, c2);
        let (l10, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[4], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[5], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[4], c0, c1, c2);
        let (l11, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[5], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[6], b[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[5], c0, c1, c2);
        let (l12, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[6], b[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[7], b[6], c0, c1, c2);
        let (l13, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = muladd_fast(a[7], b[7], c0, c1);
        let (l14, c0, c1) = (c0, c1, 0);
        debug_assert!(c1 == 0);
        let l15 = c0;

        Self(U512::from_words([
            l0, l1, l2, l3, l4, l5, l6, l7, l8, l9, l10, l11, l12, l13, l14, l15,
        ]))
    }

    /// Multiplies `a` by `b` (without modulo reduction) divide the result by `2^shift`
    /// (rounding to the nearest integer).
    /// Variable time in respect to `shift`.
    pub(crate) fn mul_shift_vartime(a: &Scalar, b: &Scalar, shift: usize) -> Scalar {
        debug_assert!(shift >= 256);

        let l = Self::mul_wide(a, b).0.to_words();
        let shiftlimbs = shift >> 5;
        let shiftlow = shift & 0x1F;
        let shifthigh = 32 - shiftlow;

        let r0 = if shift < 512 {
            let lo = l[shiftlimbs] >> shiftlow;
            let hi = if shift < 480 && shiftlow != 0 {
                l[1 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r1 = if shift < 480 {
            let lo = l[1 + shiftlimbs] >> shiftlow;
            let hi = if shift < 448 && shiftlow != 0 {
                l[2 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r2 = if shift < 448 {
            let lo = l[2 + shiftlimbs] >> shiftlow;
            let hi = if shift < 416 && shiftlow != 0 {
                l[3 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r3 = if shift < 416 {
            let lo = l[3 + shiftlimbs] >> shiftlow;
            let hi = if shift < 384 && shiftlow != 0 {
                l[4 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r4 = if shift < 384 {
            let lo = l[4 + shiftlimbs] >> shiftlow;
            let hi = if shift < 352 && shiftlow != 0 {
                l[5 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r5 = if shift < 352 {
            let lo = l[5 + shiftlimbs] >> shiftlow;
            let hi = if shift < 320 && shiftlow != 0 {
                l[6 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r6 = if shift < 320 {
            let lo = l[6 + shiftlimbs] >> shiftlow;
            let hi = if shift < 288 && shiftlow != 0 {
                l[7 + shiftlimbs] << shifthigh
            } else {
                0
            };
            hi | lo
        } else {
            0
        };

        let r7 = if shift < 288 {
            l[7 + shiftlimbs] >> shiftlow
        } else {
            0
        };

        let res = Scalar(U256::from_words([r0, r1, r2, r3, r4, r5, r6, r7]));

        // Check the highmost discarded bit and round up if it is set.
        let c = (l[(shift - 1) >> 5] >> ((shift - 1) & 0x1f)) & 1;
        Scalar::conditional_select(&res, &res.add(&Scalar::ONE), Choice::from(c as u8))
    }

    pub(super) fn reduce_impl(&self, modulus_minus_one: bool) -> Scalar {
        let neg_modulus0 = if modulus_minus_one {
            NEG_MODULUS[0] + 1
        } else {
            NEG_MODULUS[0]
        };
        let modulus = if modulus_minus_one {
            ORDER.wrapping_sub(&U256::ONE)
        } else {
            ORDER
        };

        let w = self.0.to_words();
        let n0 = w[8];
        let n1 = w[9];
        let n2 = w[10];
        let n3 = w[11];
        let n4 = w[12];
        let n5 = w[13];
        let n6 = w[14];
        let n7 = w[15];

        // 96 bit accumulator.
        //
        // Reduce 512 bits into 385.
        // m[0..12] = l[0..7] + n[0..7] * NEG_MODULUS.
        let c0 = w[0];
        let c1 = 0;
        let c2 = 0;
        let (c0, c1) = muladd_fast(n0, neg_modulus0, c0, c1);
        let (m0, c0, c1) = (c0, c1, 0);
        let (c0, c1) = sumadd_fast(w[1], c0, c1);
        let (c0, c1, c2) = muladd(n1, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n0, NEG_MODULUS[1], c0, c1, c2);
        let (m1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n1, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n0, NEG_MODULUS[2], c0, c1, c2);
        let (m2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n1, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n0, NEG_MODULUS[3], c0, c1, c2);
        let (m3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[4], c0, c1, c2);
        let (c0, c1, c2) = muladd(n4, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n1, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n0, c0, c1, c2);
        let (m4, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[5], c0, c1, c2);
        let (c0, c1, c2) = muladd(n5, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n4, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n1, c0, c1, c2);
        let (m5, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[6], c0, c1, c2);
        let (c0, c1, c2) = muladd(n6, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n5, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n4, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n2, c0, c1, c2);
        let (m6, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[7], c0, c1, c2);
        let (c0, c1, c2) = muladd(n7, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n6, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n5, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n4, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n3, c0, c1, c2);
        let (m7, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(n7, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(n6, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n5, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n4, c0, c1, c2);
        let (m8, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(n7, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n6, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n5, c0, c1, c2);
        let (m9, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(n7, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n6, c0, c1, c2);
        let (m10, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = sumadd_fast(n7, c0, c1);
        let (m11, c0, _c1) = (c0, c1, 0);
        debug_assert!(c0 <= 1);
        let m12 = c0;

        // Reduce 385 bits into 258.
        // p[0..8] = m[0..7] + m[8..12] * NEG_MODULUS.
        let c0 = m0;
        let c1 = 0;
        let c2 = 0;
        let (c0, c1) = muladd_fast(m8, neg_modulus0, c0, c1);
        let (p0, c0, c1) = (c0, c1, 0);
        let (c0, c1) = sumadd_fast(m1, c0, c1);
        let (c0, c1, c2) = muladd(m9, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(m8, NEG_MODULUS[1], c0, c1, c2);
        let (p1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(m2, c0, c1, c2);
        let (c0, c1, c2) = muladd(m10, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(m9, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(m8, NEG_MODULUS[2], c0, c1, c2);
        let (p2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(m3, c0, c1, c2);
        let (c0, c1, c2) = muladd(m11, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(m10, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(m9, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(m8, NEG_MODULUS[3], c0, c1, c2);
        let (p3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(m4, c0, c1, c2);
        let (c0, c1, c2) = muladd(m12, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(m11, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(m10, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(m9, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(m8, c0, c1, c2);
        let (p4, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(m5, c0, c1, c2);
        let (c0, c1, c2) = muladd(m12, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(m11, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(m10, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(m9, c0, c1, c2);
        let (p5, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(m6, c0, c1, c2);
        let (c0, c1, c2) = muladd(m12, NEG_MODULUS[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(m11, NEG_MODULUS[3], c0, c1, c2);
        let (c0, c1, c2) = sumadd(m10, c0, c1, c2);
        let (p6, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = sumadd_fast(m7, c0, c1);
        let (c0, c1) = muladd_fast(m12, NEG_MODULUS[3], c0, c1);
        let (c0, c1) = sumadd_fast(m11, c0, c1);
        let (p7, c0, _c1) = (c0, c1, 0);
        let p8 = c0 + m12;
        debug_assert!(p8 <= 2);

        // Reduce 258 bits into 256.
        // r[0..7] = p[0..7] + p[8] * NEG_MODULUS.
        let mut c = p0 as u64 + (neg_modulus0 as u64) * (p8 as u64);
        let r0 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p1 as u64 + (NEG_MODULUS[1] as u64) * (p8 as u64);
        let r1 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p2 as u64 + (NEG_MODULUS[2] as u64) * (p8 as u64);
        let r2 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p3 as u64 + (NEG_MODULUS[3] as u64) * (p8 as u64);
        let r3 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p4 as u64 + p8 as u64;
        let r4 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p5 as u64;
        let r5 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p6 as u64;
        let r6 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;
        c += p7 as u64;
        let r7 = (c & 0xFFFFFFFFu64) as u32;
        c >>= 32;

        // Final reduction of r.
        let r = U256::from([r0, r1, r2, r3, r4, r5, r6, r7]);
        let (r2, underflow) = r.sbb(&modulus, Limb::ZERO);
        let high_bit = Choice::from(c as u8);
        let underflow = Choice::from((underflow.0 >> 31) as u8);
        Scalar(U256::conditional_select(&r, &r2, !underflow | high_bit))
    }

    #[inline(always)] // only used in Scalar::mul(), so won't cause binary bloat
    pub(super) fn reduce(&self) -> Scalar {
        self.reduce_impl(false)
    }

    pub(super) fn reduce_nonzero(&self) -> Scalar {
        self.reduce_impl(true) + Scalar::ONE
    }
}

/// Constant-time comparison.
#[inline(always)]
fn ct_less(a: u32, b: u32) -> u32 {
    // Do not convert to Choice since it is only used internally,
    // and we don't want loss of performance.
    (a < b) as u32
}

/// Add a to the number defined by (c0,c1,c2). c2 must never overflow.
fn sumadd(a: u32, c0: u32, c1: u32, c2: u32) -> (u32, u32, u32) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let over: u32 = if new_c0 < a { 1 } else { 0 };
    let new_c1 = c1.wrapping_add(over); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, over); // never overflows by contract
    (new_c0, new_c1, new_c2)
}

/// Add a to the number defined by (c0,c1). c1 must never overflow, c2 must be zero.
fn sumadd_fast(a: u32, c0: u32, c1: u32) -> (u32, u32) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let new_c1 = c1 + ct_less(new_c0, a); // never overflows by contract (verified the next line)
    debug_assert!((new_c1 != 0) | (new_c0 >= a));
    (new_c0, new_c1)
}

/// Add a*b to the number defined by (c0,c1,c2). c2 must never overflow.
fn muladd(a: u32, b: u32, c0: u32, c1: u32, c2: u32) -> (u32, u32, u32) {
    let t = (a as u64) * (b as u64);
    let th = (t >> 32) as u32; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u32;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + ct_less(new_c0, tl); // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1.wrapping_add(new_th); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, new_th); // never overflows by contract (verified in the next line)
    debug_assert!((new_c1 >= new_th) || (new_c2 != 0));
    (new_c0, new_c1, new_c2)
}

/// Add a*b to the number defined by (c0,c1). c1 must never overflow.
fn muladd_fast(a: u32, b: u32, c0: u32, c1: u32) -> (u32, u32) {
    let t = (a as u64) * (b as u64);
    let th = (t >> 32) as u32; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u32;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + ct_less(new_c0, tl); // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1 + new_th; // never overflows by contract (verified in the next line)
    debug_assert!(new_c1 >= new_th);
    (new_c0, new_c1)
}
