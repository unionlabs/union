// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis agora lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Field arithmetic modulo \\(p = 2\^{255} - 19\\).
//!
//! The `curve25519_dalek::field` module provides a type alias
//! `curve25519_dalek::field::FieldElement` to a field element type
//! defined in the `backend` module; either `FieldElement51` or
//! `FieldElement2625`.
//!
//! Field operations defined in terms of machine
//! operations, such as field multiplication or squaring, are defined in
//! the backend implementation.
//!
//! Field operations defined in terms of other field operations, such as
//! field inversion or square roots, are defined here.

use core::cmp::{Eq, PartialEq};

use subtle::ConditionallySelectable;
use subtle::ConditionallyNegatable;
use subtle::Choice;
use subtle::ConstantTimeEq;

use constants;
use backend;

#[cfg(feature = "fiat_u32_backend")]
pub use backend::serial::fiat_u32::field::*;
#[cfg(feature = "fiat_u64_backend")]
pub use backend::serial::fiat_u64::field::*;
/// A `FieldElement` represents an element of the field
/// \\( \mathbb Z / (2\^{255} - 19)\\).
///
/// The `FieldElement` type is an alias for one of the platform-specific
/// implementations.
/// Using formally-verified field arithmetic from fiat-crypto
#[cfg(feature = "fiat_u32_backend")]
pub type FieldElement = backend::serial::fiat_u32::field::FieldElement2625;
#[cfg(feature = "fiat_u64_backend")]
pub type FieldElement = backend::serial::fiat_u64::field::FieldElement51;

#[cfg(feature = "u64_backend")]
pub use backend::serial::u64::field::*;
/// A `FieldElement` represents an element of the field
/// \\( \mathbb Z / (2\^{255} - 19)\\).
///
/// The `FieldElement` type is an alias for one of the platform-specific
/// implementations.
#[cfg(feature = "u64_backend")]
pub type FieldElement = backend::serial::u64::field::FieldElement51;

#[cfg(feature = "u32_backend")]
pub use backend::serial::u32::field::*;
/// A `FieldElement` represents an element of the field
/// \\( \mathbb Z / (2\^{255} - 19)\\).
///
/// The `FieldElement` type is an alias for one of the platform-specific
/// implementations.
#[cfg(feature = "u32_backend")]
pub type FieldElement = backend::serial::u32::field::FieldElement2625;

impl Eq for FieldElement {}

impl PartialEq for FieldElement {
    fn eq(&self, other: &FieldElement) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}

impl ConstantTimeEq for FieldElement {
    /// Test equality between two `FieldElement`s.  Since the
    /// internal representation is not canonical, the field elements
    /// are normalized to wire format before comparison.
    fn ct_eq(&self, other: &FieldElement) -> Choice {
        self.to_bytes().ct_eq(&other.to_bytes())
    }
}

impl FieldElement {
    /// Determine if this `FieldElement` is negative, in the sense
    /// used in the ed25519 paper: `x` is negative if the low bit is
    /// set.
    ///
    /// # Return
    ///
    /// If negative, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_negative(&self) -> Choice {
        let bytes = self.to_bytes();
        (bytes[0] & 1).into()
    }

    /// Determine if this `FieldElement` is zero.
    ///
    /// # Return
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        let zero = [0u8; 32];
        let bytes = self.to_bytes();

        bytes.ct_eq(&zero)
    }

    /// Compute (self^(2^250-1), self^11), used as a helper function
    /// within invert() and pow22523().
    fn pow22501(&self) -> (FieldElement, FieldElement) {
        // Instead of managing which temporary variables are used
        // for what, we define as many as we need and leave stack
        // allocation to the compiler
        //
        // Each temporary variable t_i is of the form (self)^e_i.
        // Squaring t_i corresponds to multiplying e_i by 2,
        // so the pow2k function shifts e_i left by k places.
        // Multiplying t_i and t_j corresponds to adding e_i + e_j.
        //
        // Temporary t_i                      Nonzero bits of e_i
        //
        let t0  = self.square();           // 1         e_0 = 2^1
        let t1  = t0.square().square();    // 3         e_1 = 2^3
        let t2  = self * &t1;              // 3,0       e_2 = 2^3 + 2^0
        let t3  = &t0 * &t2;               // 3,1,0
        let t4  = t3.square();             // 4,2,1
        let t5  = &t2 * &t4;               // 4,3,2,1,0
        let t6  = t5.pow2k(5);             // 9,8,7,6,5
        let t7  = &t6 * &t5;               // 9,8,7,6,5,4,3,2,1,0
        let t8  = t7.pow2k(10);            // 19..10
        let t9  = &t8 * &t7;               // 19..0
        let t10 = t9.pow2k(20);            // 39..20
        let t11 = &t10 * &t9;              // 39..0
        let t12 = t11.pow2k(10);           // 49..10
        let t13 = &t12 * &t7;              // 49..0
        let t14 = t13.pow2k(50);           // 99..50
        let t15 = &t14 * &t13;             // 99..0
        let t16 = t15.pow2k(100);          // 199..100
        let t17 = &t16 * &t15;             // 199..0
        let t18 = t17.pow2k(50);           // 249..50
        let t19 = &t18 * &t13;             // 249..0

        (t19, t3)
    }

    /// Given a slice of public `FieldElements`, replace each with its inverse.
    ///
    /// All input `FieldElements` **MUST** be nonzero.
    #[cfg(feature = "alloc")]
    pub fn batch_invert(inputs: &mut [FieldElement]) {
        // Montgomery’s Trick and Fast Implementation of Masked AES
        // Genelle, Prouff and Quisquater
        // Section 3.2

        let n = inputs.len();
        let mut scratch = vec![FieldElement::one(); n];

        // Keep an accumulator of all of the previous products
        let mut acc = FieldElement::one();

        // Pass through the input vector, recording the previous
        // products in the scratch space
        for (input, scratch) in inputs.iter().zip(scratch.iter_mut()) {
            *scratch = acc;
            acc = &acc * input;
        }

	// acc is nonzero iff all inputs are nonzero
        assert_eq!(acc.is_zero().unwrap_u8(), 0);

        // Compute the inverse of all products
        acc = acc.invert();

        // Pass through the vector backwards to compute the inverses
        // in place
        for (input, scratch) in inputs.iter_mut().rev().zip(scratch.into_iter().rev()) {
            let tmp = &acc * input;
            *input = &acc * &scratch;
            acc = tmp;
        }
    }

    /// Given a nonzero field element, compute its inverse.
    ///
    /// The inverse is computed as self^(p-2), since
    /// x^(p-2)x = x^(p-1) = 1 (mod p).
    ///
    /// This function returns zero on input zero.
    pub fn invert(&self) -> FieldElement {
        // The bits of p-2 = 2^255 -19 -2 are 11010111111...11.
        //
        //                                 nonzero bits of exponent
        let (t19, t3) = self.pow22501();   // t19: 249..0 ; t3: 3,1,0
        let t20 = t19.pow2k(5);            // 254..5
        let t21 = &t20 * &t3;              // 254..5,3,1,0

        t21
    }

    /// Raise this field element to the power (p-5)/8 = 2^252 -3.
    fn pow_p58(&self) -> FieldElement {
        // The bits of (p-5)/8 are 101111.....11.
        //
        //                                 nonzero bits of exponent
        let (t19, _) = self.pow22501();    // 249..0
        let t20 = t19.pow2k(2);            // 251..2
        let t21 = self * &t20;             // 251..2,0

        t21
    }

    /// Given `FieldElements` `u` and `v`, compute either `sqrt(u/v)`
    /// or `sqrt(i*u/v)` in constant time.
    ///
    /// This function always returns the nonnegative square root.
    ///
    /// # Return
    ///
    /// - `(Choice(1), +sqrt(u/v))  ` if `v` is nonzero and `u/v` is square;
    /// - `(Choice(1), zero)        ` if `u` is zero;
    /// - `(Choice(0), zero)        ` if `v` is zero and `u` is nonzero;
    /// - `(Choice(0), +sqrt(i*u/v))` if `u/v` is nonsquare (so `i*u/v` is square).
    ///
    pub fn sqrt_ratio_i(u: &FieldElement, v: &FieldElement) -> (Choice, FieldElement) {
        // Using the same trick as in ed25519 decoding, we merge the
        // inversion, the square root, and the square test as follows.
        //
        // To compute sqrt(α), we can compute β = α^((p+3)/8).
        // Then β^2 = ±α, so multiplying β by sqrt(-1) if necessary
        // gives sqrt(α).
        //
        // To compute 1/sqrt(α), we observe that
        //    1/β = α^(p-1 - (p+3)/8) = α^((7p-11)/8)
        //                            = α^3 * (α^7)^((p-5)/8).
        //
        // We can therefore compute sqrt(u/v) = sqrt(u)/sqrt(v)
        // by first computing
        //    r = u^((p+3)/8) v^(p-1-(p+3)/8)
        //      = u u^((p-5)/8) v^3 (v^7)^((p-5)/8)
        //      = (uv^3) (uv^7)^((p-5)/8).
        //
        // If v is nonzero and u/v is square, then r^2 = ±u/v,
        //                                     so vr^2 = ±u.
        // If vr^2 =  u, then sqrt(u/v) = r.
        // If vr^2 = -u, then sqrt(u/v) = r*sqrt(-1).
        //
        // If v is zero, r is also zero.

        let v3 = &v.square()  * v;
        let v7 = &v3.square() * v;
        let mut r = &(u * &v3) * &(u * &v7).pow_p58();
        let check = v * &r.square();

        let i = &constants::SQRT_M1;

        let correct_sign_sqrt   = check.ct_eq(        u);
        let flipped_sign_sqrt   = check.ct_eq(     &(-u));
        let flipped_sign_sqrt_i = check.ct_eq(&(&(-u)*i));

        let r_prime = &constants::SQRT_M1 * &r;
        r.conditional_assign(&r_prime, flipped_sign_sqrt | flipped_sign_sqrt_i);

        // Choose the nonnegative square root.
        let r_is_negative = r.is_negative();
        r.conditional_negate(r_is_negative);

        let was_nonzero_square = correct_sign_sqrt | flipped_sign_sqrt;

        (was_nonzero_square, r)
    }

    /// Attempt to compute `sqrt(1/self)` in constant time.
    ///
    /// Convenience wrapper around `sqrt_ratio_i`.
    ///
    /// This function always returns the nonnegative square root.
    ///
    /// # Return
    ///
    /// - `(Choice(1), +sqrt(1/self))  ` if `self` is a nonzero square;
    /// - `(Choice(0), zero)           ` if `self` is zero;
    /// - `(Choice(0), +sqrt(i/self))  ` if `self` is a nonzero nonsquare;
    ///
    pub fn invsqrt(&self) -> (Choice, FieldElement) {
        FieldElement::sqrt_ratio_i(&FieldElement::one(), self)
    }
}

#[cfg(test)]
mod test {
    use field::*;
    use subtle::ConditionallyNegatable;

    /// Random element a of GF(2^255-19), from Sage
    /// a = 1070314506888354081329385823235218444233221\
    ///     2228051251926706380353716438957572
    static A_BYTES: [u8; 32] =
        [ 0x04, 0xfe, 0xdf, 0x98, 0xa7, 0xfa, 0x0a, 0x68,
          0x84, 0x92, 0xbd, 0x59, 0x08, 0x07, 0xa7, 0x03,
          0x9e, 0xd1, 0xf6, 0xf2, 0xe1, 0xd9, 0xe2, 0xa4,
          0xa4, 0x51, 0x47, 0x36, 0xf3, 0xc3, 0xa9, 0x17];

    /// Byte representation of a**2
    static ASQ_BYTES: [u8; 32] =
        [ 0x75, 0x97, 0x24, 0x9e, 0xe6, 0x06, 0xfe, 0xab,
          0x24, 0x04, 0x56, 0x68, 0x07, 0x91, 0x2d, 0x5d,
          0x0b, 0x0f, 0x3f, 0x1c, 0xb2, 0x6e, 0xf2, 0xe2,
          0x63, 0x9c, 0x12, 0xba, 0x73, 0x0b, 0xe3, 0x62];

    /// Byte representation of 1/a
    static AINV_BYTES: [u8; 32] =
        [0x96, 0x1b, 0xcd, 0x8d, 0x4d, 0x5e, 0xa2, 0x3a,
         0xe9, 0x36, 0x37, 0x93, 0xdb, 0x7b, 0x4d, 0x70,
         0xb8, 0x0d, 0xc0, 0x55, 0xd0, 0x4c, 0x1d, 0x7b,
         0x90, 0x71, 0xd8, 0xe9, 0xb6, 0x18, 0xe6, 0x30];

    /// Byte representation of a^((p-5)/8)
    static AP58_BYTES: [u8; 32] =
        [0x6a, 0x4f, 0x24, 0x89, 0x1f, 0x57, 0x60, 0x36,
         0xd0, 0xbe, 0x12, 0x3c, 0x8f, 0xf5, 0xb1, 0x59,
         0xe0, 0xf0, 0xb8, 0x1b, 0x20, 0xd2, 0xb5, 0x1f,
         0x15, 0x21, 0xf9, 0xe3, 0xe1, 0x61, 0x21, 0x55];

    #[test]
    fn a_mul_a_vs_a_squared_constant() {
        let a = FieldElement::from_bytes(&A_BYTES);
        let asq = FieldElement::from_bytes(&ASQ_BYTES);
        assert_eq!(asq, &a * &a);
    }

    #[test]
    fn a_square_vs_a_squared_constant() {
        let a = FieldElement::from_bytes(&A_BYTES);
        let asq = FieldElement::from_bytes(&ASQ_BYTES);
        assert_eq!(asq, a.square());
    }

    #[test]
    fn a_square2_vs_a_squared_constant() {
        let a = FieldElement::from_bytes(&A_BYTES);
        let asq = FieldElement::from_bytes(&ASQ_BYTES);
        assert_eq!(a.square2(), &asq+&asq);
    }

    #[test]
    fn a_invert_vs_inverse_of_a_constant() {
        let a    = FieldElement::from_bytes(&A_BYTES);
        let ainv = FieldElement::from_bytes(&AINV_BYTES);
        let should_be_inverse = a.invert();
        assert_eq!(ainv, should_be_inverse);
        assert_eq!(FieldElement::one(), &a * &should_be_inverse);
    }

    #[test]
    fn batch_invert_a_matches_nonbatched() {
        let a    = FieldElement::from_bytes(&A_BYTES);
        let ap58 = FieldElement::from_bytes(&AP58_BYTES);
        let asq  = FieldElement::from_bytes(&ASQ_BYTES);
        let ainv = FieldElement::from_bytes(&AINV_BYTES);
        let a2   = &a + &a;
        let a_list = vec![a, ap58, asq, ainv, a2];
        let mut ainv_list = a_list.clone();
        FieldElement::batch_invert(&mut ainv_list[..]);
        for i in 0..5 {
            assert_eq!(a_list[i].invert(), ainv_list[i]);
        }
    }

    #[test]
    fn sqrt_ratio_behavior() {
        let zero = FieldElement::zero();
        let one = FieldElement::one();
        let i = constants::SQRT_M1;
        let two = &one + &one; // 2 is nonsquare mod p.
        let four = &two + &two; // 4 is square mod p.

        // 0/0 should return (1, 0) since u is 0
        let (choice, sqrt) = FieldElement::sqrt_ratio_i(&zero, &zero);
        assert_eq!(choice.unwrap_u8(), 1);
        assert_eq!(sqrt, zero);
        assert_eq!(sqrt.is_negative().unwrap_u8(), 0);

        // 1/0 should return (0, 0) since v is 0, u is nonzero
        let (choice, sqrt) = FieldElement::sqrt_ratio_i(&one, &zero);
        assert_eq!(choice.unwrap_u8(), 0);
        assert_eq!(sqrt, zero);
        assert_eq!(sqrt.is_negative().unwrap_u8(), 0);

        // 2/1 is nonsquare, so we expect (0, sqrt(i*2))
        let (choice, sqrt) = FieldElement::sqrt_ratio_i(&two, &one);
        assert_eq!(choice.unwrap_u8(), 0);
        assert_eq!(sqrt.square(), &two * &i);
        assert_eq!(sqrt.is_negative().unwrap_u8(), 0);

        // 4/1 is square, so we expect (1, sqrt(4))
        let (choice, sqrt) = FieldElement::sqrt_ratio_i(&four, &one);
        assert_eq!(choice.unwrap_u8(), 1);
        assert_eq!(sqrt.square(), four);
        assert_eq!(sqrt.is_negative().unwrap_u8(), 0);

        // 1/4 is square, so we expect (1, 1/sqrt(4))
        let (choice, sqrt) = FieldElement::sqrt_ratio_i(&one, &four);
        assert_eq!(choice.unwrap_u8(), 1);
        assert_eq!(&sqrt.square() * &four, one);
        assert_eq!(sqrt.is_negative().unwrap_u8(), 0);
    }

    #[test]
    fn a_p58_vs_ap58_constant() {
        let a    = FieldElement::from_bytes(&A_BYTES);
        let ap58 = FieldElement::from_bytes(&AP58_BYTES);
        assert_eq!(ap58, a.pow_p58());
    }

    #[test]
    fn equality() {
        let a    = FieldElement::from_bytes(&A_BYTES);
        let ainv = FieldElement::from_bytes(&AINV_BYTES);
        assert!(a == a);
        assert!(a != ainv);
    }

    /// Notice that the last element has the high bit set, which
    /// should be ignored
    static B_BYTES: [u8;32] =
        [113, 191, 169, 143,  91, 234, 121,  15,
         241, 131, 217,  36, 230, 101,  92, 234,
           8, 208, 170, 251,  97, 127,  70, 210,
          58,  23, 166,  87, 240, 169, 184, 178];

    #[test]
    fn from_bytes_highbit_is_ignored() {
        let mut cleared_bytes = B_BYTES;
        cleared_bytes[31] &= 127u8;
        let with_highbit_set    = FieldElement::from_bytes(&B_BYTES);
        let without_highbit_set = FieldElement::from_bytes(&cleared_bytes);
        assert_eq!(without_highbit_set, with_highbit_set);
    }

    #[test]
    fn conditional_negate() {
        let       one = FieldElement::one();
        let minus_one = FieldElement::minus_one();
        let mut x = one;
        x.conditional_negate(Choice::from(1));
        assert_eq!(x, minus_one);
        x.conditional_negate(Choice::from(0));
        assert_eq!(x, minus_one);
        x.conditional_negate(Choice::from(1));
        assert_eq!(x, one);
    }

    #[test]
    fn encoding_is_canonical() {
        // Encode 1 wrongly as 1 + (2^255 - 19) = 2^255 - 18
        let one_encoded_wrongly_bytes: [u8;32] = [0xee, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];
        // Decode to a field element
        let one = FieldElement::from_bytes(&one_encoded_wrongly_bytes);
        // .. then check that the encoding is correct
        let one_bytes = one.to_bytes();
        assert_eq!(one_bytes[0], 1);
        for i in 1..32 {
            assert_eq!(one_bytes[i], 0);
        }
    }

    #[test]
    fn batch_invert_empty() {
        FieldElement::batch_invert(&mut []);
    }
}
