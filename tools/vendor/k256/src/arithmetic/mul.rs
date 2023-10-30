//! From libsecp256k1:
//!
//! The Secp256k1 curve has an endomorphism, where lambda * (x, y) = (beta * x, y), where
//! lambda is {0x53,0x63,0xad,0x4c,0xc0,0x5c,0x30,0xe0,0xa5,0x26,0x1c,0x02,0x88,0x12,0x64,0x5a,
//!         0x12,0x2e,0x22,0xea,0x20,0x81,0x66,0x78,0xdf,0x02,0x96,0x7c,0x1b,0x23,0xbd,0x72}
//!
//! "Guide to Elliptic Curve Cryptography" (Hankerson, Menezes, Vanstone) gives an algorithm
//! (algorithm 3.74) to find k1 and k2 given k, such that k1 + k2 * lambda == k mod n, and k1
//! and k2 have a small size.
//! It relies on constants a1, b1, a2, b2. These constants for the value of lambda above are:
//!
//! - a1 =      {0x30,0x86,0xd2,0x21,0xa7,0xd4,0x6b,0xcd,0xe8,0x6c,0x90,0xe4,0x92,0x84,0xeb,0x15}
//! - b1 =     -{0xe4,0x43,0x7e,0xd6,0x01,0x0e,0x88,0x28,0x6f,0x54,0x7f,0xa9,0x0a,0xbf,0xe4,0xc3}
//! - a2 = {0x01,0x14,0xca,0x50,0xf7,0xa8,0xe2,0xf3,0xf6,0x57,0xc1,0x10,0x8d,0x9d,0x44,0xcf,0xd8}
//! - b2 =      {0x30,0x86,0xd2,0x21,0xa7,0xd4,0x6b,0xcd,0xe8,0x6c,0x90,0xe4,0x92,0x84,0xeb,0x15}
//!
//! The algorithm then computes c1 = round(b1 * k / n) and c2 = round(b2 * k / n), and gives
//! k1 = k - (c1*a1 + c2*a2) and k2 = -(c1*b1 + c2*b2). Instead, we use modular arithmetic, and
//! compute k1 as k - k2 * lambda, avoiding the need for constants a1 and a2.
//!
//! g1, g2 are precomputed constants used to replace division with a rounded multiplication
//! when decomposing the scalar for an endomorphism-based point multiplication.
//!
//! The possibility of using precomputed estimates is mentioned in "Guide to Elliptic Curve
//! Cryptography" (Hankerson, Menezes, Vanstone) in section 3.5.
//!
//! The derivation is described in the paper "Efficient Software Implementation of Public-Key
//! Cryptography on Sensor Networks Using the MSP430X Microcontroller" (Gouvea, Oliveira, Lopez),
//! Section 4.3 (here we use a somewhat higher-precision estimate):
//! d = a1*b2 - b1*a2
//! g1 = round((2^384)*b2/d)
//! g2 = round((2^384)*(-b1)/d)
//!
//! (Note that 'd' is also equal to the curve order here because `[a1,b1]` and `[a2,b2]` are found
//! as outputs of the Extended Euclidean Algorithm on inputs 'order' and 'lambda').

use crate::arithmetic::{
    scalar::{Scalar, WideScalar},
    ProjectivePoint,
};
use core::ops::{Mul, MulAssign};
use elliptic_curve::{
    ops::LinearCombination,
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq},
    IsHigh,
};

/// Lookup table containing precomputed values `[p, 2p, 3p, ..., 8p]`
#[derive(Copy, Clone, Default)]
struct LookupTable([ProjectivePoint; 8]);

impl From<&ProjectivePoint> for LookupTable {
    fn from(p: &ProjectivePoint) -> Self {
        let mut points = [*p; 8];
        for j in 0..7 {
            points[j + 1] = p + &points[j];
        }
        LookupTable(points)
    }
}

impl LookupTable {
    /// Given -8 <= x <= 8, returns x * p in constant time.
    pub fn select(&self, x: i8) -> ProjectivePoint {
        debug_assert!(x >= -8);
        debug_assert!(x <= 8);

        // Compute xabs = |x|
        let xmask = x >> 7;
        let xabs = (x + xmask) ^ xmask;

        // Get an array element in constant time
        let mut t = ProjectivePoint::IDENTITY;
        for j in 1..9 {
            let c = (xabs as u8).ct_eq(&(j as u8));
            t.conditional_assign(&self.0[j - 1], c);
        }
        // Now t == |x| * p.

        let neg_mask = Choice::from((xmask & 1) as u8);
        t.conditional_assign(&-t, neg_mask);
        // Now t == x * p.

        t
    }
}

const MINUS_LAMBDA: Scalar = Scalar::from_bytes_unchecked(&[
    0xac, 0x9c, 0x52, 0xb3, 0x3f, 0xa3, 0xcf, 0x1f, 0x5a, 0xd9, 0xe3, 0xfd, 0x77, 0xed, 0x9b, 0xa4,
    0xa8, 0x80, 0xb9, 0xfc, 0x8e, 0xc7, 0x39, 0xc2, 0xe0, 0xcf, 0xc8, 0x10, 0xb5, 0x12, 0x83, 0xcf,
]);

const MINUS_B1: Scalar = Scalar::from_bytes_unchecked(&[
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xe4, 0x43, 0x7e, 0xd6, 0x01, 0x0e, 0x88, 0x28, 0x6f, 0x54, 0x7f, 0xa9, 0x0a, 0xbf, 0xe4, 0xc3,
]);

const MINUS_B2: Scalar = Scalar::from_bytes_unchecked(&[
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0x8a, 0x28, 0x0a, 0xc5, 0x07, 0x74, 0x34, 0x6d, 0xd7, 0x65, 0xcd, 0xa8, 0x3d, 0xb1, 0x56, 0x2c,
]);

const G1: Scalar = Scalar::from_bytes_unchecked(&[
    0x30, 0x86, 0xd2, 0x21, 0xa7, 0xd4, 0x6b, 0xcd, 0xe8, 0x6c, 0x90, 0xe4, 0x92, 0x84, 0xeb, 0x15,
    0x3d, 0xaa, 0x8a, 0x14, 0x71, 0xe8, 0xca, 0x7f, 0xe8, 0x93, 0x20, 0x9a, 0x45, 0xdb, 0xb0, 0x31,
]);

const G2: Scalar = Scalar::from_bytes_unchecked(&[
    0xe4, 0x43, 0x7e, 0xd6, 0x01, 0x0e, 0x88, 0x28, 0x6f, 0x54, 0x7f, 0xa9, 0x0a, 0xbf, 0xe4, 0xc4,
    0x22, 0x12, 0x08, 0xac, 0x9d, 0xf5, 0x06, 0xc6, 0x15, 0x71, 0xb4, 0xae, 0x8a, 0xc4, 0x7f, 0x71,
]);

/*
 * Proof for decompose_scalar's bounds.
 *
 * Let
 *  - epsilon1 = 2^256 * |g1/2^384 - b2/d|
 *  - epsilon2 = 2^256 * |g2/2^384 - (-b1)/d|
 *  - c1 = round(k*g1/2^384)
 *  - c2 = round(k*g2/2^384)
 *
 * Lemma 1: |c1 - k*b2/d| < 2^-1 + epsilon1
 *
 *    |c1 - k*b2/d|
 *  =
 *    |c1 - k*g1/2^384 + k*g1/2^384 - k*b2/d|
 * <=   {triangle inequality}
 *    |c1 - k*g1/2^384| + |k*g1/2^384 - k*b2/d|
 *  =
 *    |c1 - k*g1/2^384| + k*|g1/2^384 - b2/d|
 * <    {rounding in c1 and 0 <= k < 2^256}
 *    2^-1 + 2^256 * |g1/2^384 - b2/d|
 *  =   {definition of epsilon1}
 *    2^-1 + epsilon1
 *
 * Lemma 2: |c2 - k*(-b1)/d| < 2^-1 + epsilon2
 *
 *    |c2 - k*(-b1)/d|
 *  =
 *    |c2 - k*g2/2^384 + k*g2/2^384 - k*(-b1)/d|
 * <=   {triangle inequality}
 *    |c2 - k*g2/2^384| + |k*g2/2^384 - k*(-b1)/d|
 *  =
 *    |c2 - k*g2/2^384| + k*|g2/2^384 - (-b1)/d|
 * <    {rounding in c2 and 0 <= k < 2^256}
 *    2^-1 + 2^256 * |g2/2^384 - (-b1)/d|
 *  =   {definition of epsilon2}
 *    2^-1 + epsilon2
 *
 * Let
 *  - k1 = k - c1*a1 - c2*a2
 *  - k2 = - c1*b1 - c2*b2
 *
 * Lemma 3: |k1| < (a1 + a2 + 1)/2 < 2^128
 *
 *    |k1|
 *  =   {definition of k1}
 *    |k - c1*a1 - c2*a2|
 *  =   {(a1*b2 - b1*a2)/n = 1}
 *    |k*(a1*b2 - b1*a2)/n - c1*a1 - c2*a2|
 *  =
 *    |a1*(k*b2/n - c1) + a2*(k*(-b1)/n - c2)|
 * <=   {triangle inequality}
 *    a1*|k*b2/n - c1| + a2*|k*(-b1)/n - c2|
 * <    {Lemma 1 and Lemma 2}
 *    a1*(2^-1 + epslion1) + a2*(2^-1 + epsilon2)
 * <    {rounding up to an integer}
 *    (a1 + a2 + 1)/2
 * <    {rounding up to a power of 2}
 *    2^128
 *
 * Lemma 4: |k2| < (-b1 + b2)/2 + 1 < 2^128
 *
 *    |k2|
 *  =   {definition of k2}
 *    |- c1*a1 - c2*a2|
 *  =   {(b1*b2 - b1*b2)/n = 0}
 *    |k*(b1*b2 - b1*b2)/n - c1*b1 - c2*b2|
 *  =
 *    |b1*(k*b2/n - c1) + b2*(k*(-b1)/n - c2)|
 * <=   {triangle inequality}
 *    (-b1)*|k*b2/n - c1| + b2*|k*(-b1)/n - c2|
 * <    {Lemma 1 and Lemma 2}
 *    (-b1)*(2^-1 + epslion1) + b2*(2^-1 + epsilon2)
 * <    {rounding up to an integer}
 *    (-b1 + b2)/2 + 1
 * <    {rounding up to a power of 2}
 *    2^128
 *
 * Let
 *  - r2 = k2 mod n
 *  - r1 = k - r2*lambda mod n.
 *
 * Notice that r1 is defined such that r1 + r2 * lambda == k (mod n).
 *
 * Lemma 5: r1 == k1 mod n.
 *
 *    r1
 * ==   {definition of r1 and r2}
 *    k - k2*lambda
 * ==   {definition of k2}
 *    k - (- c1*b1 - c2*b2)*lambda
 * ==
 *    k + c1*b1*lambda + c2*b2*lambda
 * ==  {a1 + b1*lambda == 0 mod n and a2 + b2*lambda == 0 mod n}
 *    k - c1*a1 - c2*a2
 * ==  {definition of k1}
 *    k1
 *
 * From Lemma 3, Lemma 4, Lemma 5 and the definition of r2, we can conclude that
 *
 *  - either r1 < 2^128 or -r1 mod n < 2^128
 *  - either r2 < 2^128 or -r2 mod n < 2^128.
 *
 * Q.E.D.
 */

/// Find r1 and r2 given k, such that r1 + r2 * lambda == k mod n.
fn decompose_scalar(k: &Scalar) -> (Scalar, Scalar) {
    // these _vartime calls are constant time since the shift amount is constant
    let c1 = WideScalar::mul_shift_vartime(k, &G1, 384) * MINUS_B1;
    let c2 = WideScalar::mul_shift_vartime(k, &G2, 384) * MINUS_B2;
    let r2 = c1 + c2;
    let r1 = k + r2 * MINUS_LAMBDA;

    (r1, r2)
}

// This needs to be an object to have Default implemented for it
// (required because it's used in static_map later)
// Otherwise we could just have a function returning an array.
#[derive(Copy, Clone)]
struct Radix16Decomposition([i8; 33]);

impl Radix16Decomposition {
    /// Returns an object containing a decomposition
    /// `[a_0, ..., a_32]` such that `sum(a_j * 2^(j * 4)) == x`,
    /// and `-8 <= a_j <= 7`.
    /// Assumes `x < 2^128`.
    fn new(x: &Scalar) -> Self {
        debug_assert!((x >> 128).is_zero().unwrap_u8() == 1);

        // The resulting decomposition can be negative, so, despite the limit on `x`,
        // it can have up to 256 bits, and we need an additional byte to store the carry.
        let mut output = [0i8; 33];

        // Step 1: change radix.
        // Convert from radix 256 (bytes) to radix 16 (nibbles)
        let bytes = x.to_bytes();
        for i in 0..16 {
            output[2 * i] = (bytes[31 - i] & 0xf) as i8;
            output[2 * i + 1] = ((bytes[31 - i] >> 4) & 0xf) as i8;
        }

        // Step 2: recenter coefficients from [0,16) to [-8,8)
        for i in 0..32 {
            let carry = (output[i] + 8) >> 4;
            output[i] -= carry << 4;
            output[i + 1] += carry;
        }

        Self(output)
    }
}

impl Default for Radix16Decomposition {
    fn default() -> Self {
        Self([0i8; 33])
    }
}

/// Maps an array `x` to an array using the predicate `f`.
/// We can't use the standard `map()` because as of Rust 1.51 we cannot collect into arrays.
/// Consequently, since we cannot have an uninitialized array (without `unsafe`),
/// a default value needs to be provided.
fn static_map<T: Copy, V: Copy, const N: usize>(
    f: impl Fn(T) -> V,
    x: &[T; N],
    default: V,
) -> [V; N] {
    let mut res = [default; N];
    for i in 0..N {
        res[i] = f(x[i]);
    }
    res
}

/// Maps two arrays `x` and `y` into an array using a predicate `f` that takes two arguments.
fn static_zip_map<T: Copy, S: Copy, V: Copy, const N: usize>(
    f: impl Fn(T, S) -> V,
    x: &[T; N],
    y: &[S; N],
    default: V,
) -> [V; N] {
    let mut res = [default; N];
    for i in 0..N {
        res[i] = f(x[i], y[i]);
    }
    res
}

/// Calculates a linear combination `sum(x[i] * k[i])`, `i = 0..N`
#[inline(always)]
fn lincomb_generic<const N: usize>(xs: &[ProjectivePoint; N], ks: &[Scalar; N]) -> ProjectivePoint {
    let rs = static_map(
        |k| decompose_scalar(&k),
        ks,
        (Scalar::default(), Scalar::default()),
    );
    let r1s = static_map(|(r1, _r2)| r1, &rs, Scalar::default());
    let r2s = static_map(|(_r1, r2)| r2, &rs, Scalar::default());

    let xs_beta = static_map(|x| x.endomorphism(), xs, ProjectivePoint::default());

    let r1_signs = static_map(|r| r.is_high(), &r1s, Choice::from(0u8));
    let r2_signs = static_map(|r| r.is_high(), &r2s, Choice::from(0u8));

    let r1s_c = static_zip_map(
        |r, r_sign| Scalar::conditional_select(&r, &-r, r_sign),
        &r1s,
        &r1_signs,
        Scalar::default(),
    );
    let r2s_c = static_zip_map(
        |r, r_sign| Scalar::conditional_select(&r, &-r, r_sign),
        &r2s,
        &r2_signs,
        Scalar::default(),
    );

    let tables1 = static_zip_map(
        |x, r_sign| LookupTable::from(&ProjectivePoint::conditional_select(&x, &-x, r_sign)),
        xs,
        &r1_signs,
        LookupTable::default(),
    );
    let tables2 = static_zip_map(
        |x, r_sign| LookupTable::from(&ProjectivePoint::conditional_select(&x, &-x, r_sign)),
        &xs_beta,
        &r2_signs,
        LookupTable::default(),
    );

    let digits1 = static_map(
        |r| Radix16Decomposition::new(&r),
        &r1s_c,
        Radix16Decomposition::default(),
    );
    let digits2 = static_map(
        |r| Radix16Decomposition::new(&r),
        &r2s_c,
        Radix16Decomposition::default(),
    );

    let mut acc = ProjectivePoint::IDENTITY;
    for component in 0..N {
        acc += &tables1[component].select(digits1[component].0[32]);
        acc += &tables2[component].select(digits2[component].0[32]);
    }

    for i in (0..32).rev() {
        for _j in 0..4 {
            acc = acc.double();
        }

        for component in 0..N {
            acc += &tables1[component].select(digits1[component].0[i]);
            acc += &tables2[component].select(digits2[component].0[i]);
        }
    }
    acc
}

#[inline(always)]
fn mul(x: &ProjectivePoint, k: &Scalar) -> ProjectivePoint {
    lincomb_generic(&[*x], &[*k])
}

impl LinearCombination for ProjectivePoint {
    fn lincomb(
        x: &ProjectivePoint,
        k: &Scalar,
        y: &ProjectivePoint,
        l: &Scalar,
    ) -> ProjectivePoint {
        lincomb_generic(&[*x, *y], &[*k, *l])
    }
}

impl Mul<Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, other: Scalar) -> ProjectivePoint {
        mul(&self, &other)
    }
}

impl Mul<&Scalar> for &ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, other: &Scalar) -> ProjectivePoint {
        mul(self, other)
    }
}

impl Mul<&Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, other: &Scalar) -> ProjectivePoint {
        mul(&self, other)
    }
}

impl MulAssign<Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = mul(self, &rhs);
    }
}

impl MulAssign<&Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, rhs: &Scalar) {
        *self = mul(self, rhs);
    }
}

#[cfg(test)]
mod tests {
    use crate::arithmetic::{ProjectivePoint, Scalar};
    use elliptic_curve::{ops::LinearCombination, rand_core::OsRng, Field, Group};

    #[test]
    fn test_lincomb() {
        let x = ProjectivePoint::random(&mut OsRng);
        let y = ProjectivePoint::random(&mut OsRng);
        let k = Scalar::random(&mut OsRng);
        let l = Scalar::random(&mut OsRng);

        let reference = &x * &k + &y * &l;
        let test = ProjectivePoint::lincomb(&x, &k, &y, &l);
        assert_eq!(reference, test);
    }
}
