//! Field element modulo the curve internal modulus using 32-bit limbs.
//! Inspired by the implementation in <https://github.com/bitcoin-core/secp256k1>

use crate::FieldBytes;
use elliptic_curve::{
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption},
    zeroize::Zeroize,
};

/// Scalars modulo SECP256k1 modulus (2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1).
/// Uses 10 32-bit limbs (little-endian), where in the normalized form
/// first 9 contain 26 bits of the value each, and the last one contains 22 bits.
/// ProjectiveArithmetic operations can be done without modulo reduction for some time,
/// using the remaining overflow bits.
#[derive(Clone, Copy, Debug)]
pub struct FieldElement10x26(pub(crate) [u32; 10]);

impl FieldElement10x26 {
    /// Returns the zero element.
    pub const fn zero() -> Self {
        Self([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    /// Returns the multiplicative identity.
    pub const fn one() -> Self {
        Self([1, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    /// Attempts to parse the given byte array as an SEC1-encoded field element.
    /// Does not check the result for being in the correct range.
    pub(crate) const fn from_bytes_unchecked(bytes: &[u8; 32]) -> Self {
        let w0 = (bytes[31] as u32)
            | ((bytes[30] as u32) << 8)
            | ((bytes[29] as u32) << 16)
            | (((bytes[28] & 0x3) as u32) << 24);
        let w1 = (((bytes[28] >> 2) as u32) & 0x3f)
            | ((bytes[27] as u32) << 6)
            | ((bytes[26] as u32) << 14)
            | (((bytes[25] & 0xf) as u32) << 22);
        let w2 = (((bytes[25] >> 4) as u32) & 0xf)
            | ((bytes[24] as u32) << 4)
            | ((bytes[23] as u32) << 12)
            | (((bytes[22] & 0x3f) as u32) << 20);
        let w3 = (((bytes[22] >> 6) as u32) & 0x3)
            | ((bytes[21] as u32) << 2)
            | ((bytes[20] as u32) << 10)
            | ((bytes[19] as u32) << 18);
        let w4 = (bytes[18] as u32)
            | ((bytes[17] as u32) << 8)
            | ((bytes[16] as u32) << 16)
            | (((bytes[15] & 0x3) as u32) << 24);
        let w5 = (((bytes[15] >> 2) as u32) & 0x3f)
            | ((bytes[14] as u32) << 6)
            | ((bytes[13] as u32) << 14)
            | (((bytes[12] & 0xf) as u32) << 22);
        let w6 = (((bytes[12] >> 4) as u32) & 0xf)
            | ((bytes[11] as u32) << 4)
            | ((bytes[10] as u32) << 12)
            | (((bytes[9] & 0x3f) as u32) << 20);
        let w7 = (((bytes[9] >> 6) as u32) & 0x3)
            | ((bytes[8] as u32) << 2)
            | ((bytes[7] as u32) << 10)
            | ((bytes[6] as u32) << 18);
        let w8 = (bytes[5] as u32)
            | ((bytes[4] as u32) << 8)
            | ((bytes[3] as u32) << 16)
            | (((bytes[2] & 0x3) as u32) << 24);
        let w9 = (((bytes[2] >> 2) as u32) & 0x3f)
            | ((bytes[1] as u32) << 6)
            | ((bytes[0] as u32) << 14);

        Self([w0, w1, w2, w3, w4, w5, w6, w7, w8, w9])
    }

    /// Attempts to parse the given byte array as an SEC1-encoded field element.
    ///
    /// Returns None if the byte array does not contain a big-endian integer in the range
    /// [0, p).
    pub fn from_bytes(bytes: &FieldBytes) -> CtOption<Self> {
        let res = Self::from_bytes_unchecked(bytes.as_ref());
        let overflow = res.get_overflow();

        CtOption::new(res, !overflow)
    }

    /// Returns the SEC1 encoding of this field element.
    pub fn to_bytes(self) -> FieldBytes {
        let mut r = FieldBytes::default();
        r[0] = (self.0[9] >> 14) as u8;
        r[1] = (self.0[9] >> 6) as u8;
        r[2] = ((self.0[9] as u8 & 0x3Fu8) << 2) | ((self.0[8] >> 24) as u8 & 0x3);
        r[3] = (self.0[8] >> 16) as u8;
        r[4] = (self.0[8] >> 8) as u8;
        r[5] = self.0[8] as u8;
        r[6] = (self.0[7] >> 18) as u8;
        r[7] = (self.0[7] >> 10) as u8;
        r[8] = (self.0[7] >> 2) as u8;
        r[9] = ((self.0[7] as u8 & 0x3u8) << 6) | ((self.0[6] >> 20) as u8 & 0x3fu8);
        r[10] = (self.0[6] >> 12) as u8;
        r[11] = (self.0[6] >> 4) as u8;
        r[12] = ((self.0[6] as u8 & 0xfu8) << 4) | ((self.0[5] >> 22) as u8 & 0xfu8);
        r[13] = (self.0[5] >> 14) as u8;
        r[14] = (self.0[5] >> 6) as u8;
        r[15] = ((self.0[5] as u8 & 0x3fu8) << 2) | ((self.0[4] >> 24) as u8 & 0x3u8);
        r[16] = (self.0[4] >> 16) as u8;
        r[17] = (self.0[4] >> 8) as u8;
        r[18] = self.0[4] as u8;
        r[19] = (self.0[3] >> 18) as u8;
        r[20] = (self.0[3] >> 10) as u8;
        r[21] = (self.0[3] >> 2) as u8;
        r[22] = ((self.0[3] as u8 & 0x3u8) << 6) | ((self.0[2] >> 20) as u8 & 0x3fu8);
        r[23] = (self.0[2] >> 12) as u8;
        r[24] = (self.0[2] >> 4) as u8;
        r[25] = ((self.0[2] as u8 & 0xfu8) << 4) | ((self.0[1] >> 22) as u8 & 0xfu8);
        r[26] = (self.0[1] >> 14) as u8;
        r[27] = (self.0[1] >> 6) as u8;
        r[28] = ((self.0[1] as u8 & 0x3fu8) << 2) | ((self.0[0] >> 24) as u8 & 0x3u8);
        r[29] = (self.0[0] >> 16) as u8;
        r[30] = (self.0[0] >> 8) as u8;
        r[31] = self.0[0] as u8;
        r
    }

    /// Adds `x * (2^256 - modulus)`.
    fn add_modulus_correction(&self, x: u32) -> Self {
        // add (2^256 - modulus) * x to the first limb
        let t0 = self.0[0] + x * 0x3D1u32;

        // Propagate excess bits up the limbs
        let t1 = self.0[1] + (x << 6); // add `x` times the high bit of correction (2^32)
        let t1 = t1 + (t0 >> 26);
        let t0 = t0 & 0x3FFFFFFu32;

        let t2 = self.0[2] + (t1 >> 26);
        let t1 = t1 & 0x3FFFFFFu32;

        let t3 = self.0[3] + (t2 >> 26);
        let t2 = t2 & 0x3FFFFFFu32;

        let t4 = self.0[4] + (t3 >> 26);
        let t3 = t3 & 0x3FFFFFFu32;

        let t5 = self.0[5] + (t4 >> 26);
        let t4 = t4 & 0x3FFFFFFu32;

        let t6 = self.0[6] + (t5 >> 26);
        let t5 = t5 & 0x3FFFFFFu32;

        let t7 = self.0[7] + (t6 >> 26);
        let t6 = t6 & 0x3FFFFFFu32;

        let t8 = self.0[8] + (t7 >> 26);
        let t7 = t7 & 0x3FFFFFFu32;

        let t9 = self.0[9] + (t8 >> 26);
        let t8 = t8 & 0x3FFFFFFu32;

        Self([t0, t1, t2, t3, t4, t5, t6, t7, t8, t9])
    }

    /// Subtracts the overflow in the last limb and return it with the new field element.
    /// Equivalent to subtracting a multiple of 2^256.
    fn subtract_modulus_approximation(&self) -> (Self, u32) {
        let x = self.0[9] >> 22;
        let t9 = self.0[9] & 0x03FFFFFu32; // equivalent to self -= 2^256 * x
        (
            Self([
                self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6],
                self.0[7], self.0[8], t9,
            ]),
            x,
        )
    }

    /// Checks if the field element is greater or equal to the modulus.
    fn get_overflow(&self) -> Choice {
        let m = self.0[2] & self.0[3] & self.0[4] & self.0[5] & self.0[6] & self.0[7] & self.0[8];
        let x = (self.0[9] >> 22 != 0)
            | ((self.0[9] == 0x3FFFFFu32)
                & (m == 0x3FFFFFFu32)
                & ((self.0[1] + 0x40u32 + ((self.0[0] + 0x3D1u32) >> 26)) > 0x3FFFFFFu32));
        Choice::from(x as u8)
    }

    /// Brings the field element's magnitude to 1, but does not necessarily normalize it.
    pub fn normalize_weak(&self) -> Self {
        // Reduce t9 at the start so there will be at most a single carry from the first pass
        let (t, x) = self.subtract_modulus_approximation();

        // The first pass ensures the magnitude is 1, ...
        let res = t.add_modulus_correction(x);

        // ... except for a possible carry at bit 22 of t9 (i.e. bit 256 of the field element)
        debug_assert!(res.0[9] >> 23 == 0);

        res
    }

    /// Fully normalizes the field element.
    /// That is, first nine limbs are at most 26 bit large, the last limb is at most 22 bit large,
    /// and the value is less than the modulus.
    pub fn normalize(&self) -> Self {
        let res = self.normalize_weak();

        // At most a single final reduction is needed;
        // check if the value is >= the field characteristic
        let overflow = res.get_overflow();

        // Apply the final reduction (for constant-time behaviour, we do it always)
        let res_corrected = res.add_modulus_correction(1u32);
        // Mask off the possible multiple of 2^256 from the final reduction
        let (res_corrected, x) = res_corrected.subtract_modulus_approximation();

        // If the last limb didn't carry to bit 23 already,
        // then it should have after any final reduction
        debug_assert!(x == (overflow.unwrap_u8() as u32));

        Self::conditional_select(&res, &res_corrected, overflow)
    }

    /// Checks if the field element becomes zero if normalized.
    pub fn normalizes_to_zero(&self) -> Choice {
        let res = self.normalize_weak();

        let t0 = res.0[0];
        let t1 = res.0[1];
        let t2 = res.0[2];
        let t3 = res.0[3];
        let t4 = res.0[4];
        let t5 = res.0[5];
        let t6 = res.0[6];
        let t7 = res.0[7];
        let t8 = res.0[8];
        let t9 = res.0[9];

        // z0 tracks a possible raw value of 0, z1 tracks a possible raw value of the modulus
        let z0 = t0 | t1 | t2 | t3 | t4 | t5 | t6 | t7 | t8 | t9;
        let z1 = (t0 ^ 0x3D0u32)
            & (t1 ^ 0x40u32)
            & t2
            & t3
            & t4
            & t5
            & t6
            & t7
            & t8
            & (t9 ^ 0x3C00000u32);

        Choice::from(((z0 == 0) | (z1 == 0x3FFFFFFu32)) as u8)
    }

    /// Determine if this `FieldElement10x26` is zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        Choice::from(
            ((self.0[0]
                | self.0[1]
                | self.0[2]
                | self.0[3]
                | self.0[4]
                | self.0[5]
                | self.0[6]
                | self.0[7]
                | self.0[8]
                | self.0[9])
                == 0) as u8,
        )
    }

    /// Determine if this `FieldElement10x26` is odd in the SEC1 sense: `self mod 2 == 1`.
    ///
    /// # Returns
    ///
    /// If odd, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_odd(&self) -> Choice {
        (self.0[0] as u8 & 1).into()
    }

    // The maximum number `m` for which `0x3FFFFFF * 2 * (m + 1) < 2^32`
    #[cfg(debug_assertions)]
    pub const fn max_magnitude() -> u32 {
        31u32
    }

    /// Returns -self, treating it as a value of given magnitude.
    /// The provided magnitude must be equal or greater than the actual magnitude of `self`.
    pub const fn negate(&self, magnitude: u32) -> Self {
        let m: u32 = magnitude + 1;
        let r0 = 0x3FFFC2Fu32 * 2 * m - self.0[0];
        let r1 = 0x3FFFFBFu32 * 2 * m - self.0[1];
        let r2 = 0x3FFFFFFu32 * 2 * m - self.0[2];
        let r3 = 0x3FFFFFFu32 * 2 * m - self.0[3];
        let r4 = 0x3FFFFFFu32 * 2 * m - self.0[4];
        let r5 = 0x3FFFFFFu32 * 2 * m - self.0[5];
        let r6 = 0x3FFFFFFu32 * 2 * m - self.0[6];
        let r7 = 0x3FFFFFFu32 * 2 * m - self.0[7];
        let r8 = 0x3FFFFFFu32 * 2 * m - self.0[8];
        let r9 = 0x03FFFFFu32 * 2 * m - self.0[9];
        Self([r0, r1, r2, r3, r4, r5, r6, r7, r8, r9])
    }

    /// Returns self + rhs mod p.
    /// Sums the magnitudes.
    pub const fn add(&self, rhs: &Self) -> Self {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
            self.0[5] + rhs.0[5],
            self.0[6] + rhs.0[6],
            self.0[7] + rhs.0[7],
            self.0[8] + rhs.0[8],
            self.0[9] + rhs.0[9],
        ])
    }

    /// Multiplies by a single-limb integer.
    /// Multiplies the magnitude by the same value.
    pub const fn mul_single(&self, rhs: u32) -> Self {
        Self([
            self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs,
            self.0[4] * rhs,
            self.0[5] * rhs,
            self.0[6] * rhs,
            self.0[7] * rhs,
            self.0[8] * rhs,
            self.0[9] * rhs,
        ])
    }

    #[inline(always)]
    fn mul_inner(&self, rhs: &Self) -> Self {
        /*
        `square()` is just `mul()` with equal arguments. Rust compiler is smart enough
        to do all the necessary optimizations for this case, but it needs to have this information
        inside a function. If a function is just *called* with the same arguments,
        this information cannot be used, so the function must be inlined while using the same arguments.

        Now `mul()` is quite long and therefore expensive to inline. So we have an inner (inlined)
        function, that is used inside `mul()` and `square()`, and when it is used with the same
        arguments in `square()`, compiler is able to use that fact after inlining.
        */

        let m = 0x3FFFFFFu64;
        let rr0 = 0x3D10u64;
        let rr1 = 0x400u64;

        let a0 = self.0[0] as u64;
        let a1 = self.0[1] as u64;
        let a2 = self.0[2] as u64;
        let a3 = self.0[3] as u64;
        let a4 = self.0[4] as u64;
        let a5 = self.0[5] as u64;
        let a6 = self.0[6] as u64;
        let a7 = self.0[7] as u64;
        let a8 = self.0[8] as u64;
        let a9 = self.0[9] as u64;

        let b0 = rhs.0[0] as u64;
        let b1 = rhs.0[1] as u64;
        let b2 = rhs.0[2] as u64;
        let b3 = rhs.0[3] as u64;
        let b4 = rhs.0[4] as u64;
        let b5 = rhs.0[5] as u64;
        let b6 = rhs.0[6] as u64;
        let b7 = rhs.0[7] as u64;
        let b8 = rhs.0[8] as u64;
        let b9 = rhs.0[9] as u64;

        // [... a b c] is a shorthand for ... + a<<52 + b<<26 + c<<0 mod n.
        // for 0 <= x <= 9, px is a shorthand for sum(a[i]*b[x-i], i=0..x).
        // for 9 <= x <= 18, px is a shorthand for sum(a[i]*b[x-i], i=(x-9)..9)
        // Note that [x 0 0 0 0 0 0 0 0 0 0] = [x*rr1 x*rr0].

        let mut c: u64;
        let mut d: u64;

        d = a0 * b9
            + a1 * b8
            + a2 * b7
            + a3 * b6
            + a4 * b5
            + a5 * b4
            + a6 * b3
            + a7 * b2
            + a8 * b1
            + a9 * b0;
        // [d 0 0 0 0 0 0 0 0 0] = [p9 0 0 0 0 0 0 0 0 0]
        let t9 = (d & m) as u32;
        d >>= 26;
        debug_assert!(t9 >> 26 == 0);
        debug_assert!(d >> 38 == 0);
        // [d t9 0 0 0 0 0 0 0 0 0] = [p9 0 0 0 0 0 0 0 0 0]

        c = a0 * b0;
        debug_assert!(c >> 60 == 0);
        // [d t9 0 0 0 0 0 0 0 0 c] = [p9 0 0 0 0 0 0 0 0 p0]
        d +=
            a1 * b9 + a2 * b8 + a3 * b7 + a4 * b6 + a5 * b5 + a6 * b4 + a7 * b3 + a8 * b2 + a9 * b1;
        debug_assert!(d >> 63 == 0);
        // [d t9 0 0 0 0 0 0 0 0 c] = [p10 p9 0 0 0 0 0 0 0 0 p0]
        let u0 = (d & m) as u32;
        d >>= 26;
        c += u0 as u64 * rr0;
        debug_assert!(u0 >> 26 == 0);
        debug_assert!(d >> 37 == 0);
        debug_assert!(c >> 61 == 0);
        // [d u0 t9 0 0 0 0 0 0 0 0 c-u0*rr0] = [p10 p9 0 0 0 0 0 0 0 0 p0]
        let t0 = (c & m) as u32;
        c >>= 26;
        c += u0 as u64 * rr1;
        debug_assert!(t0 >> 26 == 0);
        debug_assert!(c >> 37 == 0);
        // [d u0 t9 0 0 0 0 0 0 0 c-u0*rr1 t0-u0*rr0] = [p10 p9 0 0 0 0 0 0 0 0 p0]
        // [d 0 t9 0 0 0 0 0 0 0 c t0] = [p10 p9 0 0 0 0 0 0 0 0 p0]

        c += a0 * b1 + a1 * b0;
        debug_assert!(c >> 62 == 0);
        // [d 0 t9 0 0 0 0 0 0 0 c t0] = [p10 p9 0 0 0 0 0 0 0 p1 p0]
        d += a2 * b9 + a3 * b8 + a4 * b7 + a5 * b6 + a6 * b5 + a7 * b4 + a8 * b3 + a9 * b2;
        debug_assert!(d >> 63 == 0);
        // [d 0 t9 0 0 0 0 0 0 0 c t0] = [p11 p10 p9 0 0 0 0 0 0 0 p1 p0]
        let u1 = (d & m) as u32;
        d >>= 26;
        c += u1 as u64 * rr0;
        debug_assert!(u1 >> 26 == 0);
        debug_assert!(d >> 37 == 0);
        debug_assert!(c >> 63 == 0);
        // [d u1 0 t9 0 0 0 0 0 0 0 c-u1*rr0 t0] = [p11 p10 p9 0 0 0 0 0 0 0 p1 p0]
        let t1 = (c & m) as u32;
        c >>= 26;
        c += u1 as u64 * rr1;
        debug_assert!(t1 >> 26 == 0);
        debug_assert!(c >> 38 == 0);
        // [d u1 0 t9 0 0 0 0 0 0 c-u1*rr1 t1-u1*rr0 t0] = [p11 p10 p9 0 0 0 0 0 0 0 p1 p0]
        // [d 0 0 t9 0 0 0 0 0 0 c t1 t0] = [p11 p10 p9 0 0 0 0 0 0 0 p1 p0]

        c += a0 * b2 + a1 * b1 + a2 * b0;
        debug_assert!(c >> 62 == 0);
        // [d 0 0 t9 0 0 0 0 0 0 c t1 t0] = [p11 p10 p9 0 0 0 0 0 0 p2 p1 p0]
        d += a3 * b9 + a4 * b8 + a5 * b7 + a6 * b6 + a7 * b5 + a8 * b4 + a9 * b3;
        debug_assert!(d >> 63 == 0);
        // [d 0 0 t9 0 0 0 0 0 0 c t1 t0] = [p12 p11 p10 p9 0 0 0 0 0 0 p2 p1 p0]
        let u2 = (d & m) as u32;
        d >>= 26;
        c += u2 as u64 * rr0;
        debug_assert!(u2 >> 26 == 0);
        debug_assert!(d >> 37 == 0);
        debug_assert!(c >> 63 == 0);
        // [d u2 0 0 t9 0 0 0 0 0 0 c-u2*rr0 t1 t0] = [p12 p11 p10 p9 0 0 0 0 0 0 p2 p1 p0]
        let t2 = (c & m) as u32;
        c >>= 26;
        c += u2 as u64 * rr1;
        debug_assert!(t2 >> 26 == 0);
        debug_assert!(c >> 38 == 0);
        // [d u2 0 0 t9 0 0 0 0 0 c-u2*rr1 t2-u2*rr0 t1 t0] = [p12 p11 p10 p9 0 0 0 0 0 0 p2 p1 p0]
        // [d 0 0 0 t9 0 0 0 0 0 c t2 t1 t0] = [p12 p11 p10 p9 0 0 0 0 0 0 p2 p1 p0]

        c += a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0;
        debug_assert!(c >> 63 == 0);
        // [d 0 0 0 t9 0 0 0 0 0 c t2 t1 t0] = [p12 p11 p10 p9 0 0 0 0 0 p3 p2 p1 p0]
        d += a4 * b9 + a5 * b8 + a6 * b7 + a7 * b6 + a8 * b5 + a9 * b4;
        debug_assert!(d >> 63 == 0);
        // [d 0 0 0 t9 0 0 0 0 0 c t2 t1 t0] = [p13 p12 p11 p10 p9 0 0 0 0 0 p3 p2 p1 p0]
        let u3 = (d & m) as u32;
        d >>= 26;
        c += u3 as u64 * rr0;
        debug_assert!(u3 >> 26 == 0);
        debug_assert!(d >> 37 == 0);
        // [d u3 0 0 0 t9 0 0 0 0 0 c-u3*rr0 t2 t1 t0] = [p13 p12 p11 p10 p9 0 0 0 0 0 p3 p2 p1 p0]
        let t3 = (c & m) as u32;
        c >>= 26;
        c += u3 as u64 * rr1;
        debug_assert!(t3 >> 26 == 0);
        debug_assert!(c >> 39 == 0);
        // [d u3 0 0 0 t9 0 0 0 0 c-u3*rr1 t3-u3*rr0 t2 t1 t0] = [p13 p12 p11 p10 p9 0 0 0 0 0 p3 p2 p1 p0]
        // [d 0 0 0 0 t9 0 0 0 0 c t3 t2 t1 t0] = [p13 p12 p11 p10 p9 0 0 0 0 0 p3 p2 p1 p0]

        c += a0 * b4 + a1 * b3 + a2 * b2 + a3 * b1 + a4 * b0;
        debug_assert!(c >> 63 == 0);
        // [d 0 0 0 0 t9 0 0 0 0 c t3 t2 t1 t0] = [p13 p12 p11 p10 p9 0 0 0 0 p4 p3 p2 p1 p0]
        d += a5 * b9 + a6 * b8 + a7 * b7 + a8 * b6 + a9 * b5;
        debug_assert!(d >> 62 == 0);
        // [d 0 0 0 0 t9 0 0 0 0 c t3 t2 t1 t0] = [p14 p13 p12 p11 p10 p9 0 0 0 0 p4 p3 p2 p1 p0]
        let u4 = (d & m) as u32;
        d >>= 26;
        c += u4 as u64 * rr0;
        debug_assert!(u4 >> 26 == 0);
        debug_assert!(d >> 36 == 0);
        // [d u4 0 0 0 0 t9 0 0 0 0 c-u4*rr0 t3 t2 t1 t0] = [p14 p13 p12 p11 p10 p9 0 0 0 0 p4 p3 p2 p1 p0]
        let t4 = (c & m) as u32;
        c >>= 26;
        c += u4 as u64 * rr1;
        debug_assert!(t4 >> 26 == 0);
        debug_assert!(c >> 39 == 0);
        // [d u4 0 0 0 0 t9 0 0 0 c-u4*rr1 t4-u4*rr0 t3 t2 t1 t0] = [p14 p13 p12 p11 p10 p9 0 0 0 0 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 t9 0 0 0 c t4 t3 t2 t1 t0] = [p14 p13 p12 p11 p10 p9 0 0 0 0 p4 p3 p2 p1 p0]

        c += a0 * b5 + a1 * b4 + a2 * b3 + a3 * b2 + a4 * b1 + a5 * b0;
        debug_assert!(c >> 63 == 0);
        // [d 0 0 0 0 0 t9 0 0 0 c t4 t3 t2 t1 t0] = [p14 p13 p12 p11 p10 p9 0 0 0 p5 p4 p3 p2 p1 p0]
        d += a6 * b9 + a7 * b8 + a8 * b7 + a9 * b6;
        debug_assert!(d >> 62 == 0);
        // [d 0 0 0 0 0 t9 0 0 0 c t4 t3 t2 t1 t0] = [p15 p14 p13 p12 p11 p10 p9 0 0 0 p5 p4 p3 p2 p1 p0]
        let u5 = (d & m) as u32;
        d >>= 26;
        c += u5 as u64 * rr0;
        debug_assert!(u5 >> 26 == 0);
        debug_assert!(d >> 36 == 0);
        // [d u5 0 0 0 0 0 t9 0 0 0 c-u5*rr0 t4 t3 t2 t1 t0] = [p15 p14 p13 p12 p11 p10 p9 0 0 0 p5 p4 p3 p2 p1 p0]
        let t5 = (c & m) as u32;
        c >>= 26;
        c += u5 as u64 * rr1;
        debug_assert!(t5 >> 26 == 0);
        debug_assert!(c >> 39 == 0);
        // [d u5 0 0 0 0 0 t9 0 0 c-u5*rr1 t5-u5*rr0 t4 t3 t2 t1 t0] = [p15 p14 p13 p12 p11 p10 p9 0 0 0 p5 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 0 t9 0 0 c t5 t4 t3 t2 t1 t0] = [p15 p14 p13 p12 p11 p10 p9 0 0 0 p5 p4 p3 p2 p1 p0]

        c += a0 * b6 + a1 * b5 + a2 * b4 + a3 * b3 + a4 * b2 + a5 * b1 + a6 * b0;
        debug_assert!(c >> 63 == 0);
        // [d 0 0 0 0 0 0 t9 0 0 c t5 t4 t3 t2 t1 t0] = [p15 p14 p13 p12 p11 p10 p9 0 0 p6 p5 p4 p3 p2 p1 p0]
        d += a7 * b9 + a8 * b8 + a9 * b7;
        debug_assert!(d >> 61 == 0);
        // [d 0 0 0 0 0 0 t9 0 0 c t5 t4 t3 t2 t1 t0] = [p16 p15 p14 p13 p12 p11 p10 p9 0 0 p6 p5 p4 p3 p2 p1 p0]
        let u6 = (d & m) as u32;
        d >>= 26;
        c += u6 as u64 * rr0;
        debug_assert!(u6 >> 26 == 0);
        debug_assert!(d >> 35 == 0);
        // [d u6 0 0 0 0 0 0 t9 0 0 c-u6*rr0 t5 t4 t3 t2 t1 t0] = [p16 p15 p14 p13 p12 p11 p10 p9 0 0 p6 p5 p4 p3 p2 p1 p0]
        let t6 = (c & m) as u32;
        c >>= 26;
        c += u6 as u64 * rr1;
        debug_assert!(t6 >> 26 == 0);
        debug_assert!(c >> 39 == 0);
        // [d u6 0 0 0 0 0 0 t9 0 c-u6*rr1 t6-u6*rr0 t5 t4 t3 t2 t1 t0] = [p16 p15 p14 p13 p12 p11 p10 p9 0 0 p6 p5 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 0 0 t9 0 c t6 t5 t4 t3 t2 t1 t0] = [p16 p15 p14 p13 p12 p11 p10 p9 0 0 p6 p5 p4 p3 p2 p1 p0]

        c += a0 * b7 + a1 * b6 + a2 * b5 + a3 * b4 + a4 * b3 + a5 * b2 + a6 * b1 + a7 * b0;
        debug_assert!(c <= 0x8000007C00000007u64);
        // [d 0 0 0 0 0 0 0 t9 0 c t6 t5 t4 t3 t2 t1 t0] = [p16 p15 p14 p13 p12 p11 p10 p9 0 p7 p6 p5 p4 p3 p2 p1 p0]
        d += a8 * b9 + a9 * b8;
        debug_assert!(d >> 58 == 0);
        // [d 0 0 0 0 0 0 0 t9 0 c t6 t5 t4 t3 t2 t1 t0] = [p17 p16 p15 p14 p13 p12 p11 p10 p9 0 p7 p6 p5 p4 p3 p2 p1 p0]
        let u7 = (d & m) as u32;
        d >>= 26;
        c += u7 as u64 * rr0;
        debug_assert!(u7 >> 26 == 0);
        debug_assert!(d >> 32 == 0);
        let d32 = d as u32;
        debug_assert!(c <= 0x800001703FFFC2F7u64);
        // [d u7 0 0 0 0 0 0 0 t9 0 c-u7*rr0 t6 t5 t4 t3 t2 t1 t0] = [p17 p16 p15 p14 p13 p12 p11 p10 p9 0 p7 p6 p5 p4 p3 p2 p1 p0]
        let t7 = (c & m) as u32;
        c >>= 26;
        c += u7 as u64 * rr1;
        debug_assert!(t7 >> 26 == 0);
        debug_assert!(c >> 38 == 0);
        // [d u7 0 0 0 0 0 0 0 t9 c-u7*rr1 t7-u7*rr0 t6 t5 t4 t3 t2 t1 t0] = [p17 p16 p15 p14 p13 p12 p11 p10 p9 0 p7 p6 p5 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 0 0 0 t9 c t7 t6 t5 t4 t3 t2 t1 t0] = [p17 p16 p15 p14 p13 p12 p11 p10 p9 0 p7 p6 p5 p4 p3 p2 p1 p0]

        c +=
            a0 * b8 + a1 * b7 + a2 * b6 + a3 * b5 + a4 * b4 + a5 * b3 + a6 * b2 + a7 * b1 + a8 * b0;
        debug_assert!(c <= 0x9000007B80000008u64);
        // [d 0 0 0 0 0 0 0 0 t9 c t7 t6 t5 t4 t3 t2 t1 t0] = [p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        d = d32 as u64 + a9 * b9;
        debug_assert!(d >> 57 == 0);
        // [d 0 0 0 0 0 0 0 0 t9 c t7 t6 t5 t4 t3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let u8 = (d & m) as u32;
        d >>= 26;
        c += u8 as u64 * rr0;
        debug_assert!(u8 >> 26 == 0);
        debug_assert!(d >> 31 == 0);
        let d32 = d as u32;
        debug_assert!(c <= 0x9000016FBFFFC2F8u64);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 t7 t6 t5 t4 t3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]

        let r3 = t3;
        debug_assert!(r3 >> 26 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 t7 t6 t5 t4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r4 = t4;
        debug_assert!(r4 >> 26 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 t7 t6 t5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r5 = t5;
        debug_assert!(r5 >> 26 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 t7 t6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r6 = t6;
        debug_assert!(r6 >> 26 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 t7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r7 = t7;
        debug_assert!(r7 >> 26 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9 c-u8*rr0 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]

        let r8 = (c & m) as u32;
        c >>= 26;
        c += u8 as u64 * rr1;
        debug_assert!(r8 >> 26 == 0);
        debug_assert!(c >> 39 == 0);
        // [d u8 0 0 0 0 0 0 0 0 t9+c-u8*rr1 r8-u8*rr0 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 0 0 0 0 t9+c r8 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        c += d32 as u64 * rr0 + t9 as u64;
        debug_assert!(c >> 45 == 0);
        // [d 0 0 0 0 0 0 0 0 0 c-d*rr0 r8 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r9 = (c & (m >> 4)) as u32;
        c >>= 22;
        c += d * (rr1 << 4);
        debug_assert!(r9 >> 22 == 0);
        debug_assert!(c >> 46 == 0);
        // [d 0 0 0 0 0 0 0 0 r9+((c-d*rr1<<4)<<22)-d*rr0 r8 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        // [d 0 0 0 0 0 0 0 -d*rr1 r9+(c<<22)-d*rr0 r8 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        // [r9+(c<<22) r8 r7 r6 r5 r4 r3 t2 t1 t0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]

        d = c * (rr0 >> 4) + t0 as u64;
        debug_assert!(d >> 56 == 0);
        // [r9+(c<<22) r8 r7 r6 r5 r4 r3 t2 t1 d-c*rr0>>4] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r0 = (d & m) as u32;
        d >>= 26;
        debug_assert!(r0 >> 26 == 0);
        debug_assert!(d >> 30 == 0);
        let d32 = d as u32;
        // [r9+(c<<22) r8 r7 r6 r5 r4 r3 t2 t1+d r0-c*rr0>>4] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        d = d32 as u64 + c * (rr1 >> 4) + t1 as u64;
        debug_assert!(d >> 53 == 0);
        debug_assert!(d <= 0x10000003FFFFBFu64);
        // [r9+(c<<22) r8 r7 r6 r5 r4 r3 t2 d-c*rr1>>4 r0-c*rr0>>4] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        // [r9 r8 r7 r6 r5 r4 r3 t2 d r0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r1 = (d & m) as u32;
        d >>= 26;
        debug_assert!(r1 >> 26 == 0);
        debug_assert!(d >> 27 == 0);
        let d32 = d as u32;
        debug_assert!(d <= 0x4000000u64);
        // [r9 r8 r7 r6 r5 r4 r3 t2+d r1 r0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        d = d32 as u64 + t2 as u64;
        debug_assert!(d >> 27 == 0);
        // [r9 r8 r7 r6 r5 r4 r3 d r1 r0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r2 = d as u32;
        debug_assert!(r2 >> 27 == 0);
        // [r9 r8 r7 r6 r5 r4 r3 r2 r1 r0] = [p18 p17 p16 p15 p14 p13 p12 p11 p10 p9 p8 p7 p6 p5 p4 p3 p2 p1 p0]

        Self([r0, r1, r2, r3, r4, r5, r6, r7, r8, r9])
    }

    /// Returns self * rhs mod p
    /// Brings the magnitude to 1 (but doesn't normalize the result).
    /// The magnitudes of arguments should be <= 8.
    pub fn mul(&self, rhs: &Self) -> Self {
        self.mul_inner(rhs)
    }

    /// Returns self * self
    /// Brings the magnitude to 1 (but doesn't normalize the result).
    /// The magnitudes of arguments should be <= 8.
    pub fn square(&self) -> Self {
        self.mul_inner(self)
    }
}

impl Default for FieldElement10x26 {
    fn default() -> Self {
        Self::zero()
    }
}

impl ConditionallySelectable for FieldElement10x26 {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self([
            u32::conditional_select(&a.0[0], &b.0[0], choice),
            u32::conditional_select(&a.0[1], &b.0[1], choice),
            u32::conditional_select(&a.0[2], &b.0[2], choice),
            u32::conditional_select(&a.0[3], &b.0[3], choice),
            u32::conditional_select(&a.0[4], &b.0[4], choice),
            u32::conditional_select(&a.0[5], &b.0[5], choice),
            u32::conditional_select(&a.0[6], &b.0[6], choice),
            u32::conditional_select(&a.0[7], &b.0[7], choice),
            u32::conditional_select(&a.0[8], &b.0[8], choice),
            u32::conditional_select(&a.0[9], &b.0[9], choice),
        ])
    }
}

impl ConstantTimeEq for FieldElement10x26 {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0[0].ct_eq(&other.0[0])
            & self.0[1].ct_eq(&other.0[1])
            & self.0[2].ct_eq(&other.0[2])
            & self.0[3].ct_eq(&other.0[3])
            & self.0[4].ct_eq(&other.0[4])
            & self.0[5].ct_eq(&other.0[5])
            & self.0[6].ct_eq(&other.0[6])
            & self.0[7].ct_eq(&other.0[7])
            & self.0[8].ct_eq(&other.0[8])
            & self.0[9].ct_eq(&other.0[9])
    }
}

impl Zeroize for FieldElement10x26 {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::FieldElement10x26;

    #[test]
    fn overflow_check_after_weak_normalize() {
        // A regression test for a missing condition in `get_overflow()`.
        //
        // In `normalize()`, after the `normalize_weak()` call,
        // the excess bit from the limb 0 is propagated all the way to the last limb.
        // This constitutes an overflow, since the last bit becomes equal to (1 << 22),
        // that is 23 bits in total.
        // When `get_overflow()` is called afterwards, this was not detected,
        // since the corresponding condition (checking for the last limb being > 22 bits)
        // was missing.
        // This resulted in a debug assert firing later.
        //
        // This is essentially 2^256
        let z = FieldElement10x26([
            (1 << 26), // an excess bit here
            // the remaining full-sized limbs are at top normalized capacity
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            // the last limb is also at top normalized capacity
            (1 << 22) - 1,
        ]);

        // Used to fail here (debug_assert firing because overflow happened at an unexpected place):
        let z_normalized = z.normalize();

        // Properly normalized result, just to be sure
        // The initial number is 2^256, so the result is 0x1000003d1
        let z_reference = FieldElement10x26([0x3d1, 0x40, 0, 0, 0, 0, 0, 0, 0, 0]);

        assert_eq!(z_normalized.0, z_reference.0);
    }
}
