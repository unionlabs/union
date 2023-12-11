//! Field element modulo the curve internal modulus using 32-bit limbs.
//! Inspired by the implementation in <https://github.com/bitcoin-core/secp256k1>

use crate::FieldBytes;
use elliptic_curve::{
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption},
    zeroize::Zeroize,
};

/// Scalars modulo SECP256k1 modulus (2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1).
/// Uses 5 64-bit limbs (little-endian), where in the normalized form
/// first 4 contain 52 bits of the value each, and the last one contains 48 bits.
/// ProjectiveArithmetic operations can be done without modulo reduction for some time,
/// using the remaining overflow bits.
#[derive(Clone, Copy, Debug)]
pub struct FieldElement5x52(pub(crate) [u64; 5]);

impl FieldElement5x52 {
    /// Returns the zero element.
    pub const fn zero() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    /// Returns the multiplicative identity.
    pub const fn one() -> Self {
        Self([1, 0, 0, 0, 0])
    }

    /// Attempts to parse the given byte array as an SEC1-encoded field element.
    /// Does not check the result for being in the correct range.
    pub(crate) const fn from_bytes_unchecked(bytes: &[u8; 32]) -> Self {
        let w0 = (bytes[31] as u64)
            | ((bytes[30] as u64) << 8)
            | ((bytes[29] as u64) << 16)
            | ((bytes[28] as u64) << 24)
            | ((bytes[27] as u64) << 32)
            | ((bytes[26] as u64) << 40)
            | (((bytes[25] & 0xFu8) as u64) << 48);

        let w1 = ((bytes[25] >> 4) as u64)
            | ((bytes[24] as u64) << 4)
            | ((bytes[23] as u64) << 12)
            | ((bytes[22] as u64) << 20)
            | ((bytes[21] as u64) << 28)
            | ((bytes[20] as u64) << 36)
            | ((bytes[19] as u64) << 44);

        let w2 = (bytes[18] as u64)
            | ((bytes[17] as u64) << 8)
            | ((bytes[16] as u64) << 16)
            | ((bytes[15] as u64) << 24)
            | ((bytes[14] as u64) << 32)
            | ((bytes[13] as u64) << 40)
            | (((bytes[12] & 0xFu8) as u64) << 48);

        let w3 = ((bytes[12] >> 4) as u64)
            | ((bytes[11] as u64) << 4)
            | ((bytes[10] as u64) << 12)
            | ((bytes[9] as u64) << 20)
            | ((bytes[8] as u64) << 28)
            | ((bytes[7] as u64) << 36)
            | ((bytes[6] as u64) << 44);

        let w4 = (bytes[5] as u64)
            | ((bytes[4] as u64) << 8)
            | ((bytes[3] as u64) << 16)
            | ((bytes[2] as u64) << 24)
            | ((bytes[1] as u64) << 32)
            | ((bytes[0] as u64) << 40);

        Self([w0, w1, w2, w3, w4])
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
        let mut ret = FieldBytes::default();
        ret[0] = (self.0[4] >> 40) as u8;
        ret[1] = (self.0[4] >> 32) as u8;
        ret[2] = (self.0[4] >> 24) as u8;
        ret[3] = (self.0[4] >> 16) as u8;
        ret[4] = (self.0[4] >> 8) as u8;
        ret[5] = self.0[4] as u8;
        ret[6] = (self.0[3] >> 44) as u8;
        ret[7] = (self.0[3] >> 36) as u8;
        ret[8] = (self.0[3] >> 28) as u8;
        ret[9] = (self.0[3] >> 20) as u8;
        ret[10] = (self.0[3] >> 12) as u8;
        ret[11] = (self.0[3] >> 4) as u8;
        ret[12] = ((self.0[2] >> 48) as u8 & 0xFu8) | ((self.0[3] as u8 & 0xFu8) << 4);
        ret[13] = (self.0[2] >> 40) as u8;
        ret[14] = (self.0[2] >> 32) as u8;
        ret[15] = (self.0[2] >> 24) as u8;
        ret[16] = (self.0[2] >> 16) as u8;
        ret[17] = (self.0[2] >> 8) as u8;
        ret[18] = self.0[2] as u8;
        ret[19] = (self.0[1] >> 44) as u8;
        ret[20] = (self.0[1] >> 36) as u8;
        ret[21] = (self.0[1] >> 28) as u8;
        ret[22] = (self.0[1] >> 20) as u8;
        ret[23] = (self.0[1] >> 12) as u8;
        ret[24] = (self.0[1] >> 4) as u8;
        ret[25] = ((self.0[0] >> 48) as u8 & 0xFu8) | ((self.0[1] as u8 & 0xFu8) << 4);
        ret[26] = (self.0[0] >> 40) as u8;
        ret[27] = (self.0[0] >> 32) as u8;
        ret[28] = (self.0[0] >> 24) as u8;
        ret[29] = (self.0[0] >> 16) as u8;
        ret[30] = (self.0[0] >> 8) as u8;
        ret[31] = self.0[0] as u8;
        ret
    }

    /// Adds `x * (2^256 - modulus)`.
    fn add_modulus_correction(&self, x: u64) -> Self {
        // add (2^256 - modulus) * x to the first limb
        let t0 = self.0[0] + x * 0x1000003D1u64;

        // Propagate excess bits up the limbs
        let t1 = self.0[1] + (t0 >> 52);
        let t0 = t0 & 0xFFFFFFFFFFFFFu64;

        let t2 = self.0[2] + (t1 >> 52);
        let t1 = t1 & 0xFFFFFFFFFFFFFu64;

        let t3 = self.0[3] + (t2 >> 52);
        let t2 = t2 & 0xFFFFFFFFFFFFFu64;

        let t4 = self.0[4] + (t3 >> 52);
        let t3 = t3 & 0xFFFFFFFFFFFFFu64;

        Self([t0, t1, t2, t3, t4])
    }

    /// Subtracts the overflow in the last limb and return it with the new field element.
    /// Equivalent to subtracting a multiple of 2^256.
    fn subtract_modulus_approximation(&self) -> (Self, u64) {
        let x = self.0[4] >> 48;
        let t4 = self.0[4] & 0x0FFFFFFFFFFFFu64; // equivalent to self -= 2^256 * x
        (Self([self.0[0], self.0[1], self.0[2], self.0[3], t4]), x)
    }

    /// Checks if the field element is greater or equal to the modulus.
    fn get_overflow(&self) -> Choice {
        let m = self.0[1] & self.0[2] & self.0[3];
        let x = (self.0[4] >> 48 != 0)
            | ((self.0[4] == 0x0FFFFFFFFFFFFu64)
                & (m == 0xFFFFFFFFFFFFFu64)
                & (self.0[0] >= 0xFFFFEFFFFFC2Fu64));
        Choice::from(x as u8)
    }

    /// Brings the field element's magnitude to 1, but does not necessarily normalize it.
    pub fn normalize_weak(&self) -> Self {
        // Reduce t4 at the start so there will be at most a single carry from the first pass
        let (t, x) = self.subtract_modulus_approximation();

        // The first pass ensures the magnitude is 1, ...
        let res = t.add_modulus_correction(x);

        // ... except for a possible carry at bit 48 of t4 (i.e. bit 256 of the field element)
        debug_assert!(res.0[4] >> 49 == 0);

        res
    }

    /// Fully normalizes the field element.
    /// That is, first four limbs are at most 52 bit large, the last limb is at most 48 bit large,
    /// and the value is less than the modulus.
    pub fn normalize(&self) -> Self {
        let res = self.normalize_weak();

        // At most a single final reduction is needed;
        // check if the value is >= the field characteristic
        let overflow = res.get_overflow();

        // Apply the final reduction (for constant-time behaviour, we do it always)
        let res_corrected = res.add_modulus_correction(1u64);
        // Mask off the possible multiple of 2^256 from the final reduction
        let (res_corrected, x) = res_corrected.subtract_modulus_approximation();

        // If the last limb didn't carry to bit 48 already,
        // then it should have after any final reduction
        debug_assert!(x == (overflow.unwrap_u8() as u64));

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

        // z0 tracks a possible raw value of 0, z1 tracks a possible raw value of the modulus
        let z0 = t0 | t1 | t2 | t3 | t4;
        let z1 = (t0 ^ 0x1000003D0u64) & t1 & t2 & t3 & (t4 ^ 0xF000000000000u64);

        Choice::from(((z0 == 0) | (z1 == 0xFFFFFFFFFFFFFu64)) as u8)
    }

    /// Determine if this `FieldElement5x52` is zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        Choice::from(((self.0[0] | self.0[1] | self.0[2] | self.0[3] | self.0[4]) == 0) as u8)
    }

    /// Determine if this `FieldElement5x52` is odd in the SEC1 sense: `self mod 2 == 1`.
    ///
    /// # Returns
    ///
    /// If odd, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_odd(&self) -> Choice {
        (self.0[0] as u8 & 1).into()
    }

    /// The maximum number `m` for which `0xFFFFFFFFFFFFF * 2 * (m + 1) < 2^64`
    #[cfg(debug_assertions)]
    pub const fn max_magnitude() -> u32 {
        2047u32
    }

    /// Returns -self, treating it as a value of given magnitude.
    /// The provided magnitude must be equal or greater than the actual magnitude of `self`.
    /// Raises the magnitude by 1.
    pub const fn negate(&self, magnitude: u32) -> Self {
        let m = (magnitude + 1) as u64;
        let r0 = 0xFFFFEFFFFFC2Fu64 * 2 * m - self.0[0];
        let r1 = 0xFFFFFFFFFFFFFu64 * 2 * m - self.0[1];
        let r2 = 0xFFFFFFFFFFFFFu64 * 2 * m - self.0[2];
        let r3 = 0xFFFFFFFFFFFFFu64 * 2 * m - self.0[3];
        let r4 = 0x0FFFFFFFFFFFFu64 * 2 * m - self.0[4];
        Self([r0, r1, r2, r3, r4])
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
        ])
    }

    /// Multiplies by a single-limb integer.
    /// Multiplies the magnitude by the same value.
    pub const fn mul_single(&self, rhs: u32) -> Self {
        let rhs_u64 = rhs as u64;
        Self([
            self.0[0] * rhs_u64,
            self.0[1] * rhs_u64,
            self.0[2] * rhs_u64,
            self.0[3] * rhs_u64,
            self.0[4] * rhs_u64,
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

        let a0 = self.0[0] as u128;
        let a1 = self.0[1] as u128;
        let a2 = self.0[2] as u128;
        let a3 = self.0[3] as u128;
        let a4 = self.0[4] as u128;
        let b0 = rhs.0[0] as u128;
        let b1 = rhs.0[1] as u128;
        let b2 = rhs.0[2] as u128;
        let b3 = rhs.0[3] as u128;
        let b4 = rhs.0[4] as u128;
        let m = 0xFFFFFFFFFFFFFu128;
        let r = 0x1000003D10u128;

        debug_assert!(a0 >> 56 == 0);
        debug_assert!(a1 >> 56 == 0);
        debug_assert!(a2 >> 56 == 0);
        debug_assert!(a3 >> 56 == 0);
        debug_assert!(a4 >> 52 == 0);

        debug_assert!(b0 >> 56 == 0);
        debug_assert!(b1 >> 56 == 0);
        debug_assert!(b2 >> 56 == 0);
        debug_assert!(b3 >> 56 == 0);
        debug_assert!(b4 >> 52 == 0);

        // [... a b c] is a shorthand for ... + a<<104 + b<<52 + c<<0 mod n.
        // for 0 <= x <= 4, px is a shorthand for sum(a[i]*b[x-i], i=0..x).
        // for 4 <= x <= 8, px is a shorthand for sum(a[i]*b[x-i], i=(x-4)..4)
        // Note that [x 0 0 0 0 0] = [x*r].

        let mut d = a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0;
        debug_assert!(d >> 114 == 0);
        // [d 0 0 0] = [p3 0 0 0]
        let mut c = a4 * b4;
        debug_assert!(c >> 112 == 0);
        // [c 0 0 0 0 d 0 0 0] = [p8 0 0 0 0 p3 0 0 0]
        d += (c & m) * r;
        c >>= 52;
        debug_assert!(d >> 115 == 0);
        debug_assert!(c >> 60 == 0);
        let c64 = c as u64;
        // [c 0 0 0 0 0 d 0 0 0] = [p8 0 0 0 0 p3 0 0 0]
        let t3 = (d & m) as u64;
        d >>= 52;
        debug_assert!(t3 >> 52 == 0);
        debug_assert!(d >> 63 == 0);
        let d64 = d as u64;
        // [c 0 0 0 0 d t3 0 0 0] = [p8 0 0 0 0 p3 0 0 0]

        d = d64 as u128 + a0 * b4 + a1 * b3 + a2 * b2 + a3 * b1 + a4 * b0;
        debug_assert!(d >> 115 == 0);
        // [c 0 0 0 0 d t3 0 0 0] = [p8 0 0 0 p4 p3 0 0 0]
        d += c64 as u128 * r;
        debug_assert!(d >> 116 == 0);
        // [d t3 0 0 0] = [p8 0 0 0 p4 p3 0 0 0]
        let t4 = (d & m) as u64;
        d >>= 52;
        debug_assert!(t4 >> 52 == 0);
        debug_assert!(d >> 64 == 0);
        let d64 = d as u64;
        // [d t4 t3 0 0 0] = [p8 0 0 0 p4 p3 0 0 0]
        let tx = t4 >> 48;
        let t4 = t4 & ((m as u64) >> 4);
        debug_assert!(tx >> 4 == 0);
        debug_assert!(t4 >> 48 == 0);
        // [d t4+(tx<<48) t3 0 0 0] = [p8 0 0 0 p4 p3 0 0 0]

        c = a0 * b0;
        debug_assert!(c >> 112 == 0);
        // [d t4+(tx<<48) t3 0 0 c] = [p8 0 0 0 p4 p3 0 0 p0]
        d = d64 as u128 + a1 * b4 + a2 * b3 + a3 * b2 + a4 * b1;
        debug_assert!(d >> 115 == 0);
        // [d t4+(tx<<48) t3 0 0 c] = [p8 0 0 p5 p4 p3 0 0 p0]
        let u0 = (d & m) as u64;
        d >>= 52;
        debug_assert!(u0 >> 52 == 0);
        debug_assert!(d >> 63 == 0);
        let d64 = d as u64;
        // [d u0 t4+(tx<<48) t3 0 0 c] = [p8 0 0 p5 p4 p3 0 0 p0]
        // [d 0 t4+(tx<<48)+(u0<<52) t3 0 0 c] = [p8 0 0 p5 p4 p3 0 0 p0]
        let u0 = (u0 << 4) | tx;
        debug_assert!(u0 >> 56 == 0);
        // [d 0 t4+(u0<<48) t3 0 0 c] = [p8 0 0 p5 p4 p3 0 0 p0]
        c += u0 as u128 * ((r as u64) >> 4) as u128;
        debug_assert!(c >> 115 == 0);
        // [d 0 t4 t3 0 0 c] = [p8 0 0 p5 p4 p3 0 0 p0]
        let r0 = (c & m) as u64;
        c >>= 52;
        debug_assert!(r0 >> 52 == 0);
        debug_assert!(c >> 61 == 0);
        let c64 = c as u64;
        // [d 0 t4 t3 0 c r0] = [p8 0 0 p5 p4 p3 0 0 p0]

        c = c64 as u128 + a0 * b1 + a1 * b0;
        debug_assert!(c >> 114 == 0);
        // [d 0 t4 t3 0 c r0] = [p8 0 0 p5 p4 p3 0 p1 p0]
        d = d64 as u128 + a2 * b4 + a3 * b3 + a4 * b2;
        debug_assert!(d >> 114 == 0);
        // [d 0 t4 t3 0 c r0] = [p8 0 p6 p5 p4 p3 0 p1 p0]
        c += (d & m) * r;
        d >>= 52;
        debug_assert!(c >> 115 == 0);
        debug_assert!(d >> 62 == 0);
        let d64 = d as u64;
        // [d 0 0 t4 t3 0 c r0] = [p8 0 p6 p5 p4 p3 0 p1 p0]
        let r1 = (c & m) as u64;
        c >>= 52;
        debug_assert!(r1 >> 52 == 0);
        debug_assert!(c >> 63 == 0);
        let c64 = c as u64;
        // [d 0 0 t4 t3 c r1 r0] = [p8 0 p6 p5 p4 p3 0 p1 p0]

        c = c64 as u128 + a0 * b2 + a1 * b1 + a2 * b0;
        debug_assert!(c >> 114 == 0);
        // [d 0 0 t4 t3 c r1 r0] = [p8 0 p6 p5 p4 p3 p2 p1 p0]
        d = d64 as u128 + a3 * b4 + a4 * b3;
        debug_assert!(d >> 114 == 0);
        // [d 0 0 t4 t3 c t1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        c += (d & m) * r;
        d >>= 52;
        debug_assert!(c >> 115 == 0);
        debug_assert!(d >> 62 == 0);
        let d64 = d as u64;
        // [d 0 0 0 t4 t3 c r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]

        // [d 0 0 0 t4 t3 c r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r2 = (c & m) as u64;
        c >>= 52;
        debug_assert!(r2 >> 52 == 0);
        debug_assert!(c >> 63 == 0);
        let c64 = c as u64;
        // [d 0 0 0 t4 t3+c r2 r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        c = c64 as u128 + (d64 as u128) * r + t3 as u128;
        debug_assert!(c >> 100 == 0);
        // [t4 c r2 r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r3 = (c & m) as u64;
        c >>= 52;
        debug_assert!(r3 >> 52 == 0);
        debug_assert!(c >> 48 == 0);
        let c64 = c as u64;
        // [t4+c r3 r2 r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        c = c64 as u128 + t4 as u128;
        debug_assert!(c >> 49 == 0);
        // [c r3 r2 r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]
        let r4 = c as u64;
        debug_assert!(r4 >> 49 == 0);
        // [r4 r3 r2 r1 r0] = [p8 p7 p6 p5 p4 p3 p2 p1 p0]

        Self([r0, r1, r2, r3, r4])
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

impl Default for FieldElement5x52 {
    fn default() -> Self {
        Self::zero()
    }
}

impl ConditionallySelectable for FieldElement5x52 {
    fn conditional_select(
        a: &FieldElement5x52,
        b: &FieldElement5x52,
        choice: Choice,
    ) -> FieldElement5x52 {
        FieldElement5x52([
            u64::conditional_select(&a.0[0], &b.0[0], choice),
            u64::conditional_select(&a.0[1], &b.0[1], choice),
            u64::conditional_select(&a.0[2], &b.0[2], choice),
            u64::conditional_select(&a.0[3], &b.0[3], choice),
            u64::conditional_select(&a.0[4], &b.0[4], choice),
        ])
    }
}

impl ConstantTimeEq for FieldElement5x52 {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0[0].ct_eq(&other.0[0])
            & self.0[1].ct_eq(&other.0[1])
            & self.0[2].ct_eq(&other.0[2])
            & self.0[3].ct_eq(&other.0[3])
            & self.0[4].ct_eq(&other.0[4])
    }
}

impl Zeroize for FieldElement5x52 {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::FieldElement5x52;

    #[test]
    fn overflow_check_after_weak_normalize() {
        // A regression test for a missing condition in `get_overflow()`.
        // The condition was only missing in the 32-bit case,
        // but we're adding a 64-bit testcase nevertheless.
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
        let z = FieldElement5x52([
            (1 << 52), // an excess bit here
            // the remaining full-sized limbs are at top normalized capacity
            (1 << 52) - 1,
            (1 << 52) - 1,
            (1 << 52) - 1,
            // the last limb is also at top normalized capacity
            (1 << 48) - 1,
        ]);

        // Used to fail here (debug_assert firing because overflow happened at an unexpected place):
        let z_normalized = z.normalize();

        // Properly normalized result, just to be sure
        // The initial number is 2^256, so the result is 0x1000003d1
        let z_reference = FieldElement5x52([0x1000003d1, 0, 0, 0, 0]);

        assert_eq!(z_normalized.0, z_reference.0);
    }
}
