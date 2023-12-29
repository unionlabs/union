//! Field arithmetic modulo p = 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1

#![allow(clippy::assign_op_pattern, clippy::op_ref)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        mod field_10x26;
    } else if #[cfg(target_pointer_width = "64")] {
        mod field_5x52;
    } else {
        compile_error!("unsupported target word size (i.e. target_pointer_width)");
    }
}

cfg_if! {
    if #[cfg(debug_assertions)] {
        mod field_impl;
        use field_impl::FieldElementImpl;
    } else {
        cfg_if! {
            if #[cfg(target_pointer_width = "32")] {
                use field_10x26::FieldElement10x26 as FieldElementImpl;
            } else if #[cfg(target_pointer_width = "64")] {
                use field_5x52::FieldElement5x52 as FieldElementImpl;
            } else {
                compile_error!("unsupported target word size (i.e. target_pointer_width)");
            }
        }
    }
}

use crate::FieldBytes;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use elliptic_curve::{
    ff::Field,
    rand_core::RngCore,
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption},
    zeroize::DefaultIsZeroes,
};

#[cfg(test)]
use num_bigint::{BigUint, ToBigUint};

/// An element in the finite field used for curve coordinates.
#[derive(Clone, Copy, Debug)]
pub struct FieldElement(FieldElementImpl);

impl Field for FieldElement {
    fn random(mut rng: impl RngCore) -> Self {
        let mut bytes = FieldBytes::default();

        loop {
            rng.fill_bytes(&mut bytes);
            if let Some(fe) = Self::from_bytes(&bytes).into() {
                return fe;
            }
        }
    }

    fn zero() -> Self {
        Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    #[must_use]
    fn square(&self) -> Self {
        self.square()
    }

    #[must_use]
    fn double(&self) -> Self {
        self.double()
    }

    fn invert(&self) -> CtOption<Self> {
        self.invert()
    }

    fn sqrt(&self) -> CtOption<Self> {
        self.sqrt()
    }
}

impl FieldElement {
    /// Zero element.
    pub const ZERO: Self = Self(FieldElementImpl::zero());

    /// Multiplicative identity.
    pub const ONE: Self = Self(FieldElementImpl::one());

    /// Determine if this `FieldElement` is zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        self.0.is_zero()
    }

    /// Determine if this `FieldElement` is even in the SEC1 sense: `self mod 2 == 0`.
    ///
    /// # Returns
    ///
    /// If even, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_even(&self) -> Choice {
        !self.0.is_odd()
    }

    /// Determine if this `FieldElement` is odd in the SEC1 sense: `self mod 2 == 1`.
    ///
    /// # Returns
    ///
    /// If odd, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_odd(&self) -> Choice {
        self.0.is_odd()
    }

    /// Attempts to parse the given byte array as an SEC1-encoded field element.
    /// Does not check the result for being in the correct range.
    pub(crate) const fn from_bytes_unchecked(bytes: &[u8; 32]) -> Self {
        Self(FieldElementImpl::from_bytes_unchecked(bytes))
    }

    /// Attempts to parse the given byte array as an SEC1-encoded field element.
    ///
    /// Returns None if the byte array does not contain a big-endian integer in the range
    /// [0, p).
    pub fn from_bytes(bytes: &FieldBytes) -> CtOption<Self> {
        FieldElementImpl::from_bytes(bytes).map(Self)
    }

    /// Returns the SEC1 encoding of this field element.
    pub fn to_bytes(self) -> FieldBytes {
        self.0.normalize().to_bytes()
    }

    /// Returns -self, treating it as a value of given magnitude.
    /// The provided magnitude must be equal or greater than the actual magnitude of `self`.
    pub fn negate(&self, magnitude: u32) -> Self {
        Self(self.0.negate(magnitude))
    }

    /// Fully normalizes the field element.
    /// Brings the magnitude to 1 and modulo reduces the value.
    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }

    /// Weakly normalizes the field element.
    /// Brings the magnitude to 1, but does not guarantee the value to be less than the modulus.
    pub fn normalize_weak(&self) -> Self {
        Self(self.0.normalize_weak())
    }

    /// Checks if the field element becomes zero if normalized.
    pub fn normalizes_to_zero(&self) -> Choice {
        self.0.normalizes_to_zero()
    }

    /// Multiplies by a single-limb integer.
    /// Multiplies the magnitude by the same value.
    pub fn mul_single(&self, rhs: u32) -> Self {
        Self(self.0.mul_single(rhs))
    }

    /// Returns 2*self.
    /// Doubles the magnitude.
    pub fn double(&self) -> Self {
        Self(self.0.add(&(self.0)))
    }

    /// Returns self * rhs mod p
    /// Brings the magnitude to 1 (but doesn't normalize the result).
    /// The magnitudes of arguments should be <= 8.
    pub fn mul(&self, rhs: &Self) -> Self {
        Self(self.0.mul(&(rhs.0)))
    }

    /// Returns self * self.
    ///
    /// Brings the magnitude to 1 (but doesn't normalize the result).
    /// The magnitudes of arguments should be <= 8.
    pub fn square(&self) -> Self {
        Self(self.0.square())
    }

    /// Raises the scalar to the power `2^k`
    fn pow2k(&self, k: usize) -> Self {
        let mut x = *self;
        for _j in 0..k {
            x = x.square();
        }
        x
    }

    /// Returns the multiplicative inverse of self, if self is non-zero.
    /// The result has magnitude 1, but is not normalized.
    pub fn invert(&self) -> CtOption<Self> {
        // The binary representation of (p - 2) has 5 blocks of 1s, with lengths in
        // { 1, 2, 22, 223 }. Use an addition chain to calculate 2^n - 1 for each block:
        // [1], [2], 3, 6, 9, 11, [22], 44, 88, 176, 220, [223]

        let x2 = self.pow2k(1).mul(self);
        let x3 = x2.pow2k(1).mul(self);
        let x6 = x3.pow2k(3).mul(&x3);
        let x9 = x6.pow2k(3).mul(&x3);
        let x11 = x9.pow2k(2).mul(&x2);
        let x22 = x11.pow2k(11).mul(&x11);
        let x44 = x22.pow2k(22).mul(&x22);
        let x88 = x44.pow2k(44).mul(&x44);
        let x176 = x88.pow2k(88).mul(&x88);
        let x220 = x176.pow2k(44).mul(&x44);
        let x223 = x220.pow2k(3).mul(&x3);

        // The final result is then assembled using a sliding window over the blocks.
        let res = x223
            .pow2k(23)
            .mul(&x22)
            .pow2k(5)
            .mul(self)
            .pow2k(3)
            .mul(&x2)
            .pow2k(2)
            .mul(self);

        CtOption::new(res, !self.normalizes_to_zero())
    }

    /// Returns the square root of self mod p, or `None` if no square root exists.
    /// The result has magnitude 1, but is not normalized.
    pub fn sqrt(&self) -> CtOption<Self> {
        /*
        Given that p is congruent to 3 mod 4, we can compute the square root of
        a mod p as the (p+1)/4'th power of a.

        As (p+1)/4 is an even number, it will have the same result for a and for
        (-a). Only one of these two numbers actually has a square root however,
        so we test at the end by squaring and comparing to the input.
        Also because (p+1)/4 is an even number, the computed square root is
        itself always a square (a ** ((p+1)/4) is the square of a ** ((p+1)/8)).
        */

        // The binary representation of (p + 1)/4 has 3 blocks of 1s, with lengths in
        // { 2, 22, 223 }. Use an addition chain to calculate 2^n - 1 for each block:
        // 1, [2], 3, 6, 9, 11, [22], 44, 88, 176, 220, [223]

        let x2 = self.pow2k(1).mul(self);
        let x3 = x2.pow2k(1).mul(self);
        let x6 = x3.pow2k(3).mul(&x3);
        let x9 = x6.pow2k(3).mul(&x3);
        let x11 = x9.pow2k(2).mul(&x2);
        let x22 = x11.pow2k(11).mul(&x11);
        let x44 = x22.pow2k(22).mul(&x22);
        let x88 = x44.pow2k(44).mul(&x44);
        let x176 = x88.pow2k(88).mul(&x88);
        let x220 = x176.pow2k(44).mul(&x44);
        let x223 = x220.pow2k(3).mul(&x3);

        // The final result is then assembled using a sliding window over the blocks.
        let res = x223.pow2k(23).mul(&x22).pow2k(6).mul(&x2).pow2k(2);

        let is_root = (res.mul(&res).negate(1) + self).normalizes_to_zero();

        // Only return Some if it's the square root.
        CtOption::new(res, is_root)
    }

    #[cfg(test)]
    pub fn modulus_as_biguint() -> BigUint {
        Self::one().negate(1).to_biguint().unwrap() + 1.to_biguint().unwrap()
    }
}

impl ConditionallySelectable for FieldElement {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(FieldElementImpl::conditional_select(&(a.0), &(b.0), choice))
    }
}

impl ConstantTimeEq for FieldElement {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&(other.0))
    }
}

impl Default for FieldElement {
    fn default() -> Self {
        Self::zero()
    }
}

impl DefaultIsZeroes for FieldElement {}

impl Eq for FieldElement {}

impl PartialEq for FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&(other.0)).into()
    }
}

impl Add<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, other: FieldElement) -> FieldElement {
        FieldElement(self.0.add(&(other.0)))
    }
}

impl Add<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.add(&(other.0)))
    }
}

impl Add<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn add(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.add(&(other.0)))
    }
}

impl AddAssign<FieldElement> for FieldElement {
    fn add_assign(&mut self, other: FieldElement) {
        *self = *self + &other;
    }
}

impl AddAssign<&FieldElement> for FieldElement {
    fn add_assign(&mut self, other: &FieldElement) {
        *self = *self + other;
    }
}

impl Sub<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn sub(self, other: FieldElement) -> FieldElement {
        self + -other
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn sub(self, other: &FieldElement) -> FieldElement {
        self + -other
    }
}

impl SubAssign<FieldElement> for FieldElement {
    fn sub_assign(&mut self, other: FieldElement) {
        *self = *self + -other;
    }
}

impl SubAssign<&FieldElement> for FieldElement {
    fn sub_assign(&mut self, other: &FieldElement) {
        *self = *self + -other;
    }
}

impl Mul<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn mul(self, other: FieldElement) -> FieldElement {
        FieldElement(self.0.mul(&(other.0)))
    }
}

impl Mul<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn mul(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.mul(&(other.0)))
    }
}

impl Mul<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.mul(&(other.0)))
    }
}

impl MulAssign<FieldElement> for FieldElement {
    fn mul_assign(&mut self, rhs: FieldElement) {
        *self = *self * &rhs;
    }
}

impl MulAssign<&FieldElement> for FieldElement {
    fn mul_assign(&mut self, rhs: &FieldElement) {
        *self = *self * rhs;
    }
}

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        self.negate(1)
    }
}

impl Neg for &FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        self.negate(1)
    }
}

#[cfg(test)]
mod tests {
    use elliptic_curve::ff::Field;
    use num_bigint::{BigUint, ToBigUint};
    use proptest::prelude::*;

    use super::FieldElement;
    use crate::{
        arithmetic::dev::{biguint_to_bytes, bytes_to_biguint},
        test_vectors::field::DBL_TEST_VECTORS,
        FieldBytes,
    };

    impl From<&BigUint> for FieldElement {
        fn from(x: &BigUint) -> Self {
            let bytes = biguint_to_bytes(x);
            Self::from_bytes(&bytes.into()).unwrap()
        }
    }

    impl ToBigUint for FieldElement {
        fn to_biguint(&self) -> Option<BigUint> {
            Some(bytes_to_biguint(self.to_bytes().as_ref()))
        }
    }

    #[test]
    fn zero_is_additive_identity() {
        let zero = FieldElement::zero();
        let one = FieldElement::one();
        assert_eq!((zero + &zero).normalize(), zero);
        assert_eq!((one + &zero).normalize(), one);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let one = FieldElement::one();
        assert_eq!((one * &one).normalize(), one);
    }

    #[test]
    fn from_bytes() {
        assert_eq!(
            FieldElement::from_bytes(&FieldBytes::default()).unwrap(),
            FieldElement::zero()
        );
        assert_eq!(
            FieldElement::from_bytes(
                &[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1
                ]
                .into()
            )
            .unwrap(),
            FieldElement::one()
        );
        assert!(bool::from(
            FieldElement::from_bytes(&[0xff; 32].into()).is_none()
        ));
    }

    #[test]
    fn to_bytes() {
        assert_eq!(FieldElement::zero().to_bytes(), [0; 32].into());
        assert_eq!(
            FieldElement::one().to_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1
            ]
            .into()
        );
    }

    #[test]
    fn repeated_add() {
        let mut r = FieldElement::one();
        for i in 0..DBL_TEST_VECTORS.len() {
            assert_eq!(r.to_bytes(), DBL_TEST_VECTORS[i].into());
            r = (r + &r).normalize();
        }
    }

    #[test]
    fn repeated_double() {
        let mut r = FieldElement::one();
        for i in 0..DBL_TEST_VECTORS.len() {
            assert_eq!(r.to_bytes(), DBL_TEST_VECTORS[i].into());
            r = r.double().normalize();
        }
    }

    #[test]
    fn repeated_mul() {
        let mut r = FieldElement::one();
        let two = r + &r;
        for i in 0..DBL_TEST_VECTORS.len() {
            assert_eq!(r.normalize().to_bytes(), DBL_TEST_VECTORS[i].into());
            r = r * &two;
        }
    }

    #[test]
    fn negation() {
        let two = FieldElement::one().double();
        let neg_two = two.negate(2);
        assert_eq!((two + &neg_two).normalize(), FieldElement::zero());
        assert_eq!(neg_two.negate(3).normalize(), two.normalize());
    }

    #[test]
    fn invert() {
        assert!(bool::from(FieldElement::zero().invert().is_none()));

        let one = FieldElement::one();
        assert_eq!(one.invert().unwrap().normalize(), one);

        let two = one + &one;
        let inv_two = two.invert().unwrap();
        assert_eq!((two * &inv_two).normalize(), one);
    }

    #[test]
    fn sqrt() {
        let one = FieldElement::one();
        let two = one + &one;
        let four = two.square();
        assert_eq!(four.sqrt().unwrap().normalize(), two.normalize());
    }

    #[test]
    #[cfg_attr(
        debug_assertions,
        should_panic(expected = "assertion failed: self.normalized")
    )]
    fn unnormalized_is_odd() {
        // This is a regression test for https://github.com/RustCrypto/elliptic-curves/issues/529
        // where `is_odd()` in debug mode force-normalized its argument
        // instead of checking that it is already normalized.
        // As a result, in release (where normalization didn't happen) `is_odd()`
        // could return an incorrect value.

        let x = FieldElement::from_bytes_unchecked(&[
            61, 128, 156, 189, 241, 12, 174, 4, 80, 52, 238, 78, 188, 251, 9, 188, 95, 115, 38, 6,
            212, 168, 175, 174, 211, 232, 208, 14, 182, 45, 59, 122,
        ]);
        // Produces an unnormalized FieldElement with magnitude 1
        // (we cannot create one directly).
        let y = x.sqrt().unwrap();

        // This is fine.
        assert!(y.normalize().is_odd().unwrap_u8() == 0);

        // This panics since `y` is not normalized.
        let _result = y.is_odd().unwrap_u8();
    }

    prop_compose! {
        fn field_element()(bytes in any::<[u8; 32]>()) -> FieldElement {
            let mut res = bytes_to_biguint(&bytes);
            let m = FieldElement::modulus_as_biguint();
            // Modulus is 256 bit long, same as the maximum `res`,
            // so this is guaranteed to land us in the correct range.
            if res >= m {
                res -= m;
            }
            FieldElement::from(&res)
        }
    }

    proptest! {

        #[test]
        fn fuzzy_add(
            a in field_element(),
            b in field_element()
        ) {
            let a_bi = a.to_biguint().unwrap();
            let b_bi = b.to_biguint().unwrap();
            let res_bi = (&a_bi + &b_bi) % FieldElement::modulus_as_biguint();
            let res_ref = FieldElement::from(&res_bi);
            let res_test = (&a + &b).normalize();
            assert_eq!(res_test, res_ref);
        }

        #[test]
        fn fuzzy_mul(
            a in field_element(),
            b in field_element()
        ) {
            let a_bi = a.to_biguint().unwrap();
            let b_bi = b.to_biguint().unwrap();
            let res_bi = (&a_bi * &b_bi) % FieldElement::modulus_as_biguint();
            let res_ref = FieldElement::from(&res_bi);
            let res_test = (&a * &b).normalize();
            assert_eq!(res_test, res_ref);
        }

        #[test]
        fn fuzzy_square(
            a in field_element()
        ) {
            let a_bi = a.to_biguint().unwrap();
            let res_bi = (&a_bi * &a_bi) % FieldElement::modulus_as_biguint();
            let res_ref = FieldElement::from(&res_bi);
            let res_test = a.square().normalize();
            assert_eq!(res_test, res_ref);
        }

        #[test]
        fn fuzzy_negate(
            a in field_element()
        ) {
            let m = FieldElement::modulus_as_biguint();
            let a_bi = a.to_biguint().unwrap();
            let res_bi = (&m - &a_bi) % &m;
            let res_ref = FieldElement::from(&res_bi);
            let res_test = a.negate(1).normalize();
            assert_eq!(res_test, res_ref);
        }

        #[test]
        fn fuzzy_sqrt(
            a in field_element()
        ) {
            let m = FieldElement::modulus_as_biguint();
            let a_bi = a.to_biguint().unwrap();
            let sqr_bi = (&a_bi * &a_bi) % &m;
            let sqr = FieldElement::from(&sqr_bi);

            let res_ref1 = a;
            let possible_sqrt = (&m - &a_bi) % &m;
            let res_ref2 = FieldElement::from(&possible_sqrt);
            let res_test = sqr.sqrt().unwrap().normalize();
            // FIXME: is there a rule which square root is returned?
            assert!(res_test == res_ref1 || res_test == res_ref2);
        }

        #[test]
        fn fuzzy_invert(
            a in field_element()
        ) {
            let a = if bool::from(a.is_zero()) { FieldElement::one() } else { a };
            let a_bi = a.to_biguint().unwrap();
            let inv = a.invert().unwrap().normalize();
            let inv_bi = inv.to_biguint().unwrap();
            let m = FieldElement::modulus_as_biguint();
            assert_eq!((&inv_bi * &a_bi) % &m, 1.to_biguint().unwrap());
        }
    }
}
