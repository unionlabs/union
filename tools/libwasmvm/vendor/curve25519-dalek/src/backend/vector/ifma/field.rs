// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

#![allow(non_snake_case)]

use core::ops::{Add, Mul, Neg};
use packed_simd::{u64x4, IntoBits};

use backend::serial::u64::field::FieldElement51;

/// A wrapper around `vpmadd52luq` that works on `u64x4`.
#[inline(always)]
unsafe fn madd52lo(z: u64x4, x: u64x4, y: u64x4) -> u64x4 {
    use core::arch::x86_64::_mm256_madd52lo_epu64;
    _mm256_madd52lo_epu64(z.into_bits(), x.into_bits(), y.into_bits()).into_bits()
}

/// A wrapper around `vpmadd52huq` that works on `u64x4`.
#[inline(always)]
unsafe fn madd52hi(z: u64x4, x: u64x4, y: u64x4) -> u64x4 {
    use core::arch::x86_64::_mm256_madd52hi_epu64;
    _mm256_madd52hi_epu64(z.into_bits(), x.into_bits(), y.into_bits()).into_bits()
}

/// A vector of four field elements in radix 2^51, with unreduced coefficients.
#[derive(Copy, Clone, Debug)]
pub struct F51x4Unreduced(pub(crate) [u64x4; 5]);

/// A vector of four field elements in radix 2^51, with reduced coefficients.
#[derive(Copy, Clone, Debug)]
pub struct F51x4Reduced(pub(crate) [u64x4; 5]);

#[derive(Copy, Clone)]
pub enum Shuffle {
    AAAA,
    BBBB,
    BADC,
    BACD,
    ADDA,
    CBCB,
    ABDC,
    ABAB,
    DBBD,
    CACA,
}

#[inline(always)]
fn shuffle_lanes(x: u64x4, control: Shuffle) -> u64x4 {
    unsafe {
        use core::arch::x86_64::_mm256_permute4x64_epi64 as perm;

        match control {
            Shuffle::AAAA => perm(x.into_bits(), 0b00_00_00_00).into_bits(),
            Shuffle::BBBB => perm(x.into_bits(), 0b01_01_01_01).into_bits(),
            Shuffle::BADC => perm(x.into_bits(), 0b10_11_00_01).into_bits(),
            Shuffle::BACD => perm(x.into_bits(), 0b11_10_00_01).into_bits(),
            Shuffle::ADDA => perm(x.into_bits(), 0b00_11_11_00).into_bits(),
            Shuffle::CBCB => perm(x.into_bits(), 0b01_10_01_10).into_bits(),
            Shuffle::ABDC => perm(x.into_bits(), 0b10_11_01_00).into_bits(),
            Shuffle::ABAB => perm(x.into_bits(), 0b01_00_01_00).into_bits(),
            Shuffle::DBBD => perm(x.into_bits(), 0b11_01_01_11).into_bits(),
            Shuffle::CACA => perm(x.into_bits(), 0b00_10_00_10).into_bits(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Lanes {
    D,
    C,
    AB,
    AC,
    AD,
    BCD,
}

#[inline]
fn blend_lanes(x: u64x4, y: u64x4, control: Lanes) -> u64x4 {
    unsafe {
        use core::arch::x86_64::_mm256_blend_epi32 as blend;

        match control {
            Lanes::D => blend(x.into_bits(), y.into_bits(), 0b11_00_00_00).into_bits(),
            Lanes::C => blend(x.into_bits(), y.into_bits(), 0b00_11_00_00).into_bits(),
            Lanes::AB => blend(x.into_bits(), y.into_bits(), 0b00_00_11_11).into_bits(),
            Lanes::AC => blend(x.into_bits(), y.into_bits(), 0b00_11_00_11).into_bits(),
            Lanes::AD => blend(x.into_bits(), y.into_bits(), 0b11_00_00_11).into_bits(),
            Lanes::BCD => blend(x.into_bits(), y.into_bits(), 0b11_11_11_00).into_bits(),
        }
    }
}

impl F51x4Unreduced {
    pub fn zero() -> F51x4Unreduced {
        F51x4Unreduced([u64x4::splat(0); 5])
    }

    pub fn new(
        x0: &FieldElement51,
        x1: &FieldElement51,
        x2: &FieldElement51,
        x3: &FieldElement51,
    ) -> F51x4Unreduced {
        F51x4Unreduced([
            u64x4::new(x0.0[0], x1.0[0], x2.0[0], x3.0[0]),
            u64x4::new(x0.0[1], x1.0[1], x2.0[1], x3.0[1]),
            u64x4::new(x0.0[2], x1.0[2], x2.0[2], x3.0[2]),
            u64x4::new(x0.0[3], x1.0[3], x2.0[3], x3.0[3]),
            u64x4::new(x0.0[4], x1.0[4], x2.0[4], x3.0[4]),
        ])
    }

    pub fn split(&self) -> [FieldElement51; 4] {
        let x = &self.0;
        [
            FieldElement51([
                x[0].extract(0),
                x[1].extract(0),
                x[2].extract(0),
                x[3].extract(0),
                x[4].extract(0),
            ]),
            FieldElement51([
                x[0].extract(1),
                x[1].extract(1),
                x[2].extract(1),
                x[3].extract(1),
                x[4].extract(1),
            ]),
            FieldElement51([
                x[0].extract(2),
                x[1].extract(2),
                x[2].extract(2),
                x[3].extract(2),
                x[4].extract(2),
            ]),
            FieldElement51([
                x[0].extract(3),
                x[1].extract(3),
                x[2].extract(3),
                x[3].extract(3),
                x[4].extract(3),
            ]),
        ]
    }

    #[inline]
    pub fn diff_sum(&self) -> F51x4Unreduced {
        // tmp1 = (B, A, D, C)
        let tmp1 = self.shuffle(Shuffle::BADC);
        // tmp2 = (-A, B, -C, D)
        let tmp2 = self.blend(&self.negate_lazy(), Lanes::AC);
        // (B - A, B + A, D - C, D + C)
        tmp1 + tmp2
    }

    #[inline]
    pub fn negate_lazy(&self) -> F51x4Unreduced {
        let lo = u64x4::splat(36028797018963664u64);
        let hi = u64x4::splat(36028797018963952u64);
        F51x4Unreduced([
            lo - self.0[0],
            hi - self.0[1],
            hi - self.0[2],
            hi - self.0[3],
            hi - self.0[4],
        ])
    }

    #[inline]
    pub fn shuffle(&self, control: Shuffle) -> F51x4Unreduced {
        F51x4Unreduced([
            shuffle_lanes(self.0[0], control),
            shuffle_lanes(self.0[1], control),
            shuffle_lanes(self.0[2], control),
            shuffle_lanes(self.0[3], control),
            shuffle_lanes(self.0[4], control),
        ])
    }

    #[inline]
    pub fn blend(&self, other: &F51x4Unreduced, control: Lanes) -> F51x4Unreduced {
        F51x4Unreduced([
            blend_lanes(self.0[0], other.0[0], control),
            blend_lanes(self.0[1], other.0[1], control),
            blend_lanes(self.0[2], other.0[2], control),
            blend_lanes(self.0[3], other.0[3], control),
            blend_lanes(self.0[4], other.0[4], control),
        ])
    }
}

impl Neg for F51x4Reduced {
    type Output = F51x4Reduced;

    fn neg(self) -> F51x4Reduced {
        F51x4Unreduced::from(self).negate_lazy().into()
    }
}

use subtle::Choice;
use subtle::ConditionallySelectable;

impl ConditionallySelectable for F51x4Reduced {
    #[inline]
    fn conditional_select(a: &F51x4Reduced, b: &F51x4Reduced, choice: Choice) -> F51x4Reduced {
        let mask = (-(choice.unwrap_u8() as i64)) as u64;
        let mask_vec = u64x4::splat(mask);
        F51x4Reduced([
            a.0[0] ^ (mask_vec & (a.0[0] ^ b.0[0])),
            a.0[1] ^ (mask_vec & (a.0[1] ^ b.0[1])),
            a.0[2] ^ (mask_vec & (a.0[2] ^ b.0[2])),
            a.0[3] ^ (mask_vec & (a.0[3] ^ b.0[3])),
            a.0[4] ^ (mask_vec & (a.0[4] ^ b.0[4])),
        ])
    }

    #[inline]
    fn conditional_assign(&mut self, other: &F51x4Reduced, choice: Choice) {
        let mask = (-(choice.unwrap_u8() as i64)) as u64;
        let mask_vec = u64x4::splat(mask);
        self.0[0] ^= mask_vec & (self.0[0] ^ other.0[0]);
        self.0[1] ^= mask_vec & (self.0[1] ^ other.0[1]);
        self.0[2] ^= mask_vec & (self.0[2] ^ other.0[2]);
        self.0[3] ^= mask_vec & (self.0[3] ^ other.0[3]);
        self.0[4] ^= mask_vec & (self.0[4] ^ other.0[4]);
    }
}

impl F51x4Reduced {
    #[inline]
    pub fn shuffle(&self, control: Shuffle) -> F51x4Reduced {
        F51x4Reduced([
            shuffle_lanes(self.0[0], control),
            shuffle_lanes(self.0[1], control),
            shuffle_lanes(self.0[2], control),
            shuffle_lanes(self.0[3], control),
            shuffle_lanes(self.0[4], control),
        ])
    }

    #[inline]
    pub fn blend(&self, other: &F51x4Reduced, control: Lanes) -> F51x4Reduced {
        F51x4Reduced([
            blend_lanes(self.0[0], other.0[0], control),
            blend_lanes(self.0[1], other.0[1], control),
            blend_lanes(self.0[2], other.0[2], control),
            blend_lanes(self.0[3], other.0[3], control),
            blend_lanes(self.0[4], other.0[4], control),
        ])
    }

    #[inline]
    pub fn square(&self) -> F51x4Unreduced {
        unsafe {
            let x = &self.0;

            // Represent values with coeff. 2
            let mut z0_2 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);
            let mut z6_2 = u64x4::splat(0);
            let mut z7_2 = u64x4::splat(0);
            let mut z9_2 = u64x4::splat(0);

            // Represent values with coeff. 4
            let mut z2_4 = u64x4::splat(0);
            let mut z3_4 = u64x4::splat(0);
            let mut z4_4 = u64x4::splat(0);
            let mut z5_4 = u64x4::splat(0);
            let mut z6_4 = u64x4::splat(0);
            let mut z7_4 = u64x4::splat(0);
            let mut z8_4 = u64x4::splat(0);

            let mut z0_1 = u64x4::splat(0);
            z0_1 = madd52lo(z0_1, x[0], x[0]);

            let mut z1_1 = u64x4::splat(0);
            z1_2 = madd52lo(z1_2, x[0], x[1]);
            z1_2 = madd52hi(z1_2, x[0], x[0]);

            z2_4 = madd52hi(z2_4, x[0], x[1]);
            let mut z2_1 = z2_4 << 2;
            z2_2 = madd52lo(z2_2, x[0], x[2]);
            z2_1 = madd52lo(z2_1, x[1], x[1]);

            z3_4 = madd52hi(z3_4, x[0], x[2]);
            let mut z3_1 = z3_4 << 2;
            z3_2 = madd52lo(z3_2, x[1], x[2]);
            z3_2 = madd52lo(z3_2, x[0], x[3]);
            z3_2 = madd52hi(z3_2, x[1], x[1]);

            z4_4 = madd52hi(z4_4, x[1], x[2]);
            z4_4 = madd52hi(z4_4, x[0], x[3]);
            let mut z4_1 = z4_4 << 2;
            z4_2 = madd52lo(z4_2, x[1], x[3]);
            z4_2 = madd52lo(z4_2, x[0], x[4]);
            z4_1 = madd52lo(z4_1, x[2], x[2]);

            z5_4 = madd52hi(z5_4, x[1], x[3]);
            z5_4 = madd52hi(z5_4, x[0], x[4]);
            let mut z5_1 = z5_4 << 2;
            z5_2 = madd52lo(z5_2, x[2], x[3]);
            z5_2 = madd52lo(z5_2, x[1], x[4]);
            z5_2 = madd52hi(z5_2, x[2], x[2]);

            z6_4 = madd52hi(z6_4, x[2], x[3]);
            z6_4 = madd52hi(z6_4, x[1], x[4]);
            let mut z6_1 = z6_4 << 2;
            z6_2 = madd52lo(z6_2, x[2], x[4]);
            z6_1 = madd52lo(z6_1, x[3], x[3]);

            z7_4 = madd52hi(z7_4, x[2], x[4]);
            let mut z7_1 = z7_4 << 2;
            z7_2 = madd52lo(z7_2, x[3], x[4]);
            z7_2 = madd52hi(z7_2, x[3], x[3]);

            z8_4 = madd52hi(z8_4, x[3], x[4]);
            let mut z8_1 = z8_4 << 2;
            z8_1 = madd52lo(z8_1, x[4], x[4]);

            let mut z9_1 = u64x4::splat(0);
            z9_2 = madd52hi(z9_2, x[4], x[4]);

            z5_1 += z5_2 << 1;
            z6_1 += z6_2 << 1;
            z7_1 += z7_2 << 1;
            z9_1 += z9_2 << 1;

            let mut t0 = u64x4::splat(0);
            let mut t1 = u64x4::splat(0);
            let r19 = u64x4::splat(19);

            t0 = madd52hi(t0, r19, z9_1);
            t1 = madd52lo(t1, r19, z9_1 >> 52);

            z4_2 = madd52lo(z4_2, r19, z8_1 >> 52);
            z3_2 = madd52lo(z3_2, r19, z7_1 >> 52);
            z2_2 = madd52lo(z2_2, r19, z6_1 >> 52);
            z1_2 = madd52lo(z1_2, r19, z5_1 >> 52);

            z0_2 = madd52lo(z0_2, r19, t0 + t1);
            z1_2 = madd52hi(z1_2, r19, z5_1);
            z2_2 = madd52hi(z2_2, r19, z6_1);
            z3_2 = madd52hi(z3_2, r19, z7_1);
            z4_2 = madd52hi(z4_2, r19, z8_1);

            z0_1 = madd52lo(z0_1, r19, z5_1);
            z1_1 = madd52lo(z1_1, r19, z6_1);
            z2_1 = madd52lo(z2_1, r19, z7_1);
            z3_1 = madd52lo(z3_1, r19, z8_1);
            z4_1 = madd52lo(z4_1, r19, z9_1);

            F51x4Unreduced([
                z0_1 + z0_2 + z0_2,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

impl From<F51x4Reduced> for F51x4Unreduced {
    #[inline]
    fn from(x: F51x4Reduced) -> F51x4Unreduced {
        F51x4Unreduced(x.0)
    }
}

impl From<F51x4Unreduced> for F51x4Reduced {
    #[inline]
    fn from(x: F51x4Unreduced) -> F51x4Reduced {
        let mask = u64x4::splat((1 << 51) - 1);
        let r19 = u64x4::splat(19);

        // Compute carryouts in parallel
        let c0 = x.0[0] >> 51;
        let c1 = x.0[1] >> 51;
        let c2 = x.0[2] >> 51;
        let c3 = x.0[3] >> 51;
        let c4 = x.0[4] >> 51;

        unsafe {
            F51x4Reduced([
                madd52lo(x.0[0] & mask, c4, r19),
                (x.0[1] & mask) + c0,
                (x.0[2] & mask) + c1,
                (x.0[3] & mask) + c2,
                (x.0[4] & mask) + c3,
            ])
        }
    }
}

impl Add<F51x4Unreduced> for F51x4Unreduced {
    type Output = F51x4Unreduced;
    #[inline]
    fn add(self, rhs: F51x4Unreduced) -> F51x4Unreduced {
        F51x4Unreduced([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
        ])
    }
}

impl<'a> Mul<(u32, u32, u32, u32)> for &'a F51x4Reduced {
    type Output = F51x4Unreduced;
    #[inline]
    fn mul(self, scalars: (u32, u32, u32, u32)) -> F51x4Unreduced {
        unsafe {
            let x = &self.0;
            let y = u64x4::new(
                scalars.0 as u64,
                scalars.1 as u64,
                scalars.2 as u64,
                scalars.3 as u64,
            );
            let r19 = u64x4::splat(19);

            let mut z0_1 = u64x4::splat(0);
            let mut z1_1 = u64x4::splat(0);
            let mut z2_1 = u64x4::splat(0);
            let mut z3_1 = u64x4::splat(0);
            let mut z4_1 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);

            // Wave 0
            z4_2 = madd52hi(z4_2, y, x[3]);
            z5_2 = madd52hi(z5_2, y, x[4]);
            z4_1 = madd52lo(z4_1, y, x[4]);
            z0_1 = madd52lo(z0_1, y, x[0]);
            z3_1 = madd52lo(z3_1, y, x[3]);
            z2_1 = madd52lo(z2_1, y, x[2]);
            z1_1 = madd52lo(z1_1, y, x[1]);
            z3_2 = madd52hi(z3_2, y, x[2]);

            // Wave 2
            z2_2 = madd52hi(z2_2, y, x[1]);
            z1_2 = madd52hi(z1_2, y, x[0]);
            z0_1 = madd52lo(z0_1, z5_2 + z5_2, r19);

            F51x4Unreduced([
                z0_1,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

impl<'a, 'b> Mul<&'b F51x4Reduced> for &'a F51x4Reduced {
    type Output = F51x4Unreduced;
    #[inline]
    fn mul(self, rhs: &'b F51x4Reduced) -> F51x4Unreduced {
        unsafe {
            // Inputs
            let x = &self.0;
            let y = &rhs.0;

            // Accumulators for terms with coeff 1
            let mut z0_1 = u64x4::splat(0);
            let mut z1_1 = u64x4::splat(0);
            let mut z2_1 = u64x4::splat(0);
            let mut z3_1 = u64x4::splat(0);
            let mut z4_1 = u64x4::splat(0);
            let mut z5_1 = u64x4::splat(0);
            let mut z6_1 = u64x4::splat(0);
            let mut z7_1 = u64x4::splat(0);
            let mut z8_1 = u64x4::splat(0);

            // Accumulators for terms with coeff 2
            let mut z0_2 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);
            let mut z6_2 = u64x4::splat(0);
            let mut z7_2 = u64x4::splat(0);
            let mut z8_2 = u64x4::splat(0);
            let mut z9_2 = u64x4::splat(0);

            // LLVM doesn't seem to do much work reordering IFMA
            // instructions, so try to organize them into "waves" of 8
            // independent operations (4c latency, 0.5 c throughput
            // means 8 in flight)

            // Wave 0
            z4_1 = madd52lo(z4_1, x[2], y[2]);
            z5_2 = madd52hi(z5_2, x[2], y[2]);
            z5_1 = madd52lo(z5_1, x[4], y[1]);
            z6_2 = madd52hi(z6_2, x[4], y[1]);
            z6_1 = madd52lo(z6_1, x[4], y[2]);
            z7_2 = madd52hi(z7_2, x[4], y[2]);
            z7_1 = madd52lo(z7_1, x[4], y[3]);
            z8_2 = madd52hi(z8_2, x[4], y[3]);

            // Wave 1
            z4_1 = madd52lo(z4_1, x[3], y[1]);
            z5_2 = madd52hi(z5_2, x[3], y[1]);
            z5_1 = madd52lo(z5_1, x[3], y[2]);
            z6_2 = madd52hi(z6_2, x[3], y[2]);
            z6_1 = madd52lo(z6_1, x[3], y[3]);
            z7_2 = madd52hi(z7_2, x[3], y[3]);
            z7_1 = madd52lo(z7_1, x[3], y[4]);
            z8_2 = madd52hi(z8_2, x[3], y[4]);

            // Wave 2
            z8_1 = madd52lo(z8_1, x[4], y[4]);
            z9_2 = madd52hi(z9_2, x[4], y[4]);
            z4_1 = madd52lo(z4_1, x[4], y[0]);
            z5_2 = madd52hi(z5_2, x[4], y[0]);
            z5_1 = madd52lo(z5_1, x[2], y[3]);
            z6_2 = madd52hi(z6_2, x[2], y[3]);
            z6_1 = madd52lo(z6_1, x[2], y[4]);
            z7_2 = madd52hi(z7_2, x[2], y[4]);

            let z8 = z8_1 + z8_2 + z8_2;
            let z9 = z9_2 + z9_2;

            // Wave 3
            z3_1 = madd52lo(z3_1, x[3], y[0]);
            z4_2 = madd52hi(z4_2, x[3], y[0]);
            z4_1 = madd52lo(z4_1, x[1], y[3]);
            z5_2 = madd52hi(z5_2, x[1], y[3]);
            z5_1 = madd52lo(z5_1, x[1], y[4]);
            z6_2 = madd52hi(z6_2, x[1], y[4]);
            z2_1 = madd52lo(z2_1, x[2], y[0]);
            z3_2 = madd52hi(z3_2, x[2], y[0]);

            let z6 = z6_1 + z6_2 + z6_2;
            let z7 = z7_1 + z7_2 + z7_2;

            // Wave 4
            z3_1 = madd52lo(z3_1, x[2], y[1]);
            z4_2 = madd52hi(z4_2, x[2], y[1]);
            z4_1 = madd52lo(z4_1, x[0], y[4]);
            z5_2 = madd52hi(z5_2, x[0], y[4]);
            z1_1 = madd52lo(z1_1, x[1], y[0]);
            z2_2 = madd52hi(z2_2, x[1], y[0]);
            z2_1 = madd52lo(z2_1, x[1], y[1]);
            z3_2 = madd52hi(z3_2, x[1], y[1]);

            let z5 = z5_1 + z5_2 + z5_2;

            // Wave 5
            z3_1 = madd52lo(z3_1, x[1], y[2]);
            z4_2 = madd52hi(z4_2, x[1], y[2]);
            z0_1 = madd52lo(z0_1, x[0], y[0]);
            z1_2 = madd52hi(z1_2, x[0], y[0]);
            z1_1 = madd52lo(z1_1, x[0], y[1]);
            z2_1 = madd52lo(z2_1, x[0], y[2]);
            z2_2 = madd52hi(z2_2, x[0], y[1]);
            z3_2 = madd52hi(z3_2, x[0], y[2]);

            let mut t0 = u64x4::splat(0);
            let mut t1 = u64x4::splat(0);
            let r19 = u64x4::splat(19);

            // Wave 6
            t0 = madd52hi(t0, r19, z9);
            t1 = madd52lo(t1, r19, z9 >> 52);
            z3_1 = madd52lo(z3_1, x[0], y[3]);
            z4_2 = madd52hi(z4_2, x[0], y[3]);
            z1_2 = madd52lo(z1_2, r19, z5 >> 52);
            z2_2 = madd52lo(z2_2, r19, z6 >> 52);
            z3_2 = madd52lo(z3_2, r19, z7 >> 52);
            z0_1 = madd52lo(z0_1, r19, z5);

            // Wave 7
            z4_1 = madd52lo(z4_1, r19, z9);
            z1_1 = madd52lo(z1_1, r19, z6);
            z0_2 = madd52lo(z0_2, r19, t0 + t1);
            z4_2 = madd52hi(z4_2, r19, z8);
            z2_1 = madd52lo(z2_1, r19, z7);
            z1_2 = madd52hi(z1_2, r19, z5);
            z2_2 = madd52hi(z2_2, r19, z6);
            z3_2 = madd52hi(z3_2, r19, z7);

            // Wave 8
            z3_1 = madd52lo(z3_1, r19, z8);
            z4_2 = madd52lo(z4_2, r19, z8 >> 52);

            F51x4Unreduced([
                z0_1 + z0_2 + z0_2,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn vpmadd52luq() {
        let x = u64x4::splat(2);
        let y = u64x4::splat(3);
        let mut z = u64x4::splat(5);

        z = unsafe { madd52lo(z, x, y) };

        assert_eq!(z, u64x4::splat(5 + 2 * 3));
    }

    #[test]
    fn new_split_round_trip_on_reduced_input() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();

        let ax4 = F51x4Unreduced::new(&a, &a, &a, &a);
        let splits = ax4.split();

        for i in 0..4 {
            assert_eq!(a, splits[i]);
        }
    }

    #[test]
    fn new_split_round_trip_on_unreduced_input() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        // ... but now multiply it by 16 without reducing coeffs
        let a16 = FieldElement51([
            a.0[0] << 4,
            a.0[1] << 4,
            a.0[2] << 4,
            a.0[3] << 4,
            a.0[4] << 4,
        ]);

        let a16x4 = F51x4Unreduced::new(&a16, &a16, &a16, &a16);
        let splits = a16x4.split();

        for i in 0..4 {
            assert_eq!(a16, splits[i]);
        }
    }

    #[test]
    fn test_reduction() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        // ... but now multiply it by 128 without reducing coeffs
        let abig = FieldElement51([
            a.0[0] << 4,
            a.0[1] << 4,
            a.0[2] << 4,
            a.0[3] << 4,
            a.0[4] << 4,
        ]);

        let abigx4: F51x4Reduced = F51x4Unreduced::new(&abig, &abig, &abig, &abig).into();

        let splits = F51x4Unreduced::from(abigx4).split();
        let c = &a * &FieldElement51([(1 << 4), 0, 0, 0, 0]);

        for i in 0..4 {
            assert_eq!(c, splits[i]);
        }
    }

    #[test]
    fn mul_matches_serial() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        let b = FieldElement51([98098, 87987897, 0, 1, 0]).invert();
        let c = &a * &b;

        let ax4: F51x4Reduced = F51x4Unreduced::new(&a, &a, &a, &a).into();
        let bx4: F51x4Reduced = F51x4Unreduced::new(&b, &b, &b, &b).into();
        let cx4 = &ax4 * &bx4;

        let splits = cx4.split();

        for i in 0..4 {
            assert_eq!(c, splits[i]);
        }
    }

    #[test]
    fn iterated_mul_matches_serial() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        let b = FieldElement51([98098, 87987897, 0, 1, 0]).invert();
        let mut c = &a * &b;
        for _i in 0..1024 {
            c = &a * &c;
            c = &b * &c;
        }

        let ax4: F51x4Reduced = F51x4Unreduced::new(&a, &a, &a, &a).into();
        let bx4: F51x4Reduced = F51x4Unreduced::new(&b, &b, &b, &b).into();
        let mut cx4 = &ax4 * &bx4;
        for _i in 0..1024 {
            cx4 = &ax4 * &F51x4Reduced::from(cx4);
            cx4 = &bx4 * &F51x4Reduced::from(cx4);
        }

        let splits = cx4.split();

        for i in 0..4 {
            assert_eq!(c, splits[i]);
        }
    }

    #[test]
    fn square_matches_mul() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();

        let ax4: F51x4Reduced = F51x4Unreduced::new(&a, &a, &a, &a).into();
        let cx4 = &ax4 * &ax4;
        let cx4_sq = ax4.square();

        let splits = cx4.split();
        let splits_sq = cx4_sq.split();

        for i in 0..4 {
            assert_eq!(splits_sq[i], splits[i]);
        }
    }

    #[test]
    fn iterated_square_matches_serial() {
        // Invert a small field element to get a big one
        let mut a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        let mut ax4 = F51x4Unreduced::new(&a, &a, &a, &a);
        for _j in 0..1024 {
            a = a.square();
            ax4 = F51x4Reduced::from(ax4).square();

            let splits = ax4.split();
            for i in 0..4 {
                assert_eq!(a, splits[i]);
            }
        }
    }

    #[test]
    fn iterated_u32_mul_matches_serial() {
        // Invert a small field element to get a big one
        let a = FieldElement51([2438, 24, 243, 0, 0]).invert();
        let b = FieldElement51([121665, 0, 0, 0, 0]);
        let mut c = &a * &b;
        for _i in 0..1024 {
            c = &b * &c;
        }

        let ax4 = F51x4Unreduced::new(&a, &a, &a, &a);
        let bx4 = (121665u32, 121665u32, 121665u32, 121665u32);
        let mut cx4 = &F51x4Reduced::from(ax4) * bx4;
        for _i in 0..1024 {
            cx4 = &F51x4Reduced::from(cx4) * bx4;
        }

        let splits = cx4.split();

        for i in 0..4 {
            assert_eq!(c, splits[i]);
        }
    }

    #[test]
    fn shuffle_AAAA() {
        let x0 = FieldElement51::from_bytes(&[0x10; 32]);
        let x1 = FieldElement51::from_bytes(&[0x11; 32]);
        let x2 = FieldElement51::from_bytes(&[0x12; 32]);
        let x3 = FieldElement51::from_bytes(&[0x13; 32]);

        let x = F51x4Unreduced::new(&x0, &x1, &x2, &x3);

        let y = x.shuffle(Shuffle::AAAA);
        let splits = y.split();

        assert_eq!(splits[0], x0);
        assert_eq!(splits[1], x0);
        assert_eq!(splits[2], x0);
        assert_eq!(splits[3], x0);
    }

    #[test]
    fn blend_AB() {
        let x0 = FieldElement51::from_bytes(&[0x10; 32]);
        let x1 = FieldElement51::from_bytes(&[0x11; 32]);
        let x2 = FieldElement51::from_bytes(&[0x12; 32]);
        let x3 = FieldElement51::from_bytes(&[0x13; 32]);

        let x = F51x4Unreduced::new(&x0, &x1, &x2, &x3);
        let z = F51x4Unreduced::new(&x3, &x2, &x1, &x0);

        let y = x.blend(&z, Lanes::AB);
        let splits = y.split();

        assert_eq!(splits[0], x3);
        assert_eq!(splits[1], x2);
        assert_eq!(splits[2], x2);
        assert_eq!(splits[3], x3);
    }
}
