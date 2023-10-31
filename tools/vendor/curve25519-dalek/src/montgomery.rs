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

//! Scalar multiplication on the Montgomery form of Curve25519.
//!
//! To avoid notational confusion with the Edwards code, we use
//! variables \\( u, v \\) for the Montgomery curve, so that “Montgomery
//! \\(u\\)” here corresponds to “Montgomery \\(x\\)” elsewhere.
//!
//! Montgomery arithmetic works not on the curve itself, but on the
//! \\(u\\)-line, which discards sign information and unifies the curve
//! and its quadratic twist.  See [_Montgomery curves and their
//! arithmetic_][costello-smith] by Costello and Smith for more details.
//!
//! The `MontgomeryPoint` struct contains the affine \\(u\\)-coordinate
//! \\(u\_0(P)\\) of a point \\(P\\) on either the curve or the twist.
//! Here the map \\(u\_0 : \mathcal M \rightarrow \mathbb F\_p \\) is
//! defined by \\(u\_0((u,v)) = u\\); \\(u\_0(\mathcal O) = 0\\).  See
//! section 5.4 of Costello-Smith for more details.
//!
//! # Scalar Multiplication
//!
//! Scalar multiplication on `MontgomeryPoint`s is provided by the `*`
//! operator, which implements the Montgomery ladder.
//!
//! # Edwards Conversion
//!
//! The \\(2\\)-to-\\(1\\) map from the Edwards model to the Montgomery
//! \\(u\\)-line is provided by `EdwardsPoint::to_montgomery()`.
//!
//! To lift a `MontgomeryPoint` to an `EdwardsPoint`, use
//! `MontgomeryPoint::to_edwards()`, which takes a sign parameter.
//! This function rejects `MontgomeryPoints` which correspond to points
//! on the twist.
//!
//! [costello-smith]: https://eprint.iacr.org/2017/212.pdf

// We allow non snake_case names because coordinates in projective space are
// traditionally denoted by the capitalisation of their respective
// counterparts in affine space.  Yeah, you heard me, rustc, I'm gonna have my
// affine and projective cakes and eat both of them too.
#![allow(non_snake_case)]

use core::ops::{Mul, MulAssign};

use constants::{APLUS2_OVER_FOUR, MONTGOMERY_A, MONTGOMERY_A_NEG};
use edwards::{CompressedEdwardsY, EdwardsPoint};
use field::FieldElement;
use scalar::Scalar;

use traits::Identity;

use subtle::Choice;
use subtle::ConstantTimeEq;
use subtle::{ConditionallyNegatable, ConditionallySelectable};

use zeroize::Zeroize;

/// Holds the \\(u\\)-coordinate of a point on the Montgomery form of
/// Curve25519 or its twist.
#[derive(Copy, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MontgomeryPoint(pub [u8; 32]);

/// Equality of `MontgomeryPoint`s is defined mod p.
impl ConstantTimeEq for MontgomeryPoint {
    fn ct_eq(&self, other: &MontgomeryPoint) -> Choice {
        let self_fe = FieldElement::from_bytes(&self.0);
        let other_fe = FieldElement::from_bytes(&other.0);

        self_fe.ct_eq(&other_fe)
    }
}

impl Default for MontgomeryPoint {
    fn default() -> MontgomeryPoint {
        MontgomeryPoint([0u8; 32])
    }
}

impl PartialEq for MontgomeryPoint {
    fn eq(&self, other: &MontgomeryPoint) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}

impl Eq for MontgomeryPoint {}

impl Identity for MontgomeryPoint {
    /// Return the group identity element, which has order 4.
    fn identity() -> MontgomeryPoint {
        MontgomeryPoint([0u8; 32])
    }
}

impl Zeroize for MontgomeryPoint {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl MontgomeryPoint {
    /// View this `MontgomeryPoint` as an array of bytes.
    pub fn as_bytes<'a>(&'a self) -> &'a [u8; 32] {
        &self.0
    }

    /// Convert this `MontgomeryPoint` to an array of bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Attempt to convert to an `EdwardsPoint`, using the supplied
    /// choice of sign for the `EdwardsPoint`.
    ///
    /// # Inputs
    ///
    /// * `sign`: a `u8` donating the desired sign of the resulting
    ///   `EdwardsPoint`.  `0` denotes positive and `1` negative.
    ///
    /// # Return
    ///
    /// * `Some(EdwardsPoint)` if `self` is the \\(u\\)-coordinate of a
    /// point on (the Montgomery form of) Curve25519;
    ///
    /// * `None` if `self` is the \\(u\\)-coordinate of a point on the
    /// twist of (the Montgomery form of) Curve25519;
    ///
    pub fn to_edwards(&self, sign: u8) -> Option<EdwardsPoint> {
        // To decompress the Montgomery u coordinate to an
        // `EdwardsPoint`, we apply the birational map to obtain the
        // Edwards y coordinate, then do Edwards decompression.
        //
        // The birational map is y = (u-1)/(u+1).
        //
        // The exceptional points are the zeros of the denominator,
        // i.e., u = -1.
        //
        // But when u = -1, v^2 = u*(u^2+486662*u+1) = 486660.
        //
        // Since this is nonsquare mod p, u = -1 corresponds to a point
        // on the twist, not the curve, so we can reject it early.

        let u = FieldElement::from_bytes(&self.0);

        if u == FieldElement::minus_one() { return None; }

        let one = FieldElement::one();

        let y = &(&u - &one) * &(&u + &one).invert();

        let mut y_bytes = y.to_bytes();
        y_bytes[31] ^= sign << 7;

        CompressedEdwardsY(y_bytes).decompress()
    }
}

/// Perform the Elligator2 mapping to a Montgomery point.
///
/// See <https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-10#section-6.7.1>
//
// TODO Determine how much of the hash-to-group API should be exposed after the CFRG
//      draft gets into a more polished/accepted state.
#[allow(unused)]
pub(crate) fn elligator_encode(r_0: &FieldElement) -> MontgomeryPoint {
    let one = FieldElement::one();
    let d_1 = &one + &r_0.square2(); /* 2r^2 */

    let d = &MONTGOMERY_A_NEG * &(d_1.invert()); /* A/(1+2r^2) */

    let d_sq = &d.square();
    let au = &MONTGOMERY_A * &d;

    let inner = &(d_sq + &au) + &one;
    let eps = &d * &inner; /* eps = d^3 + Ad^2 + d */

    let (eps_is_sq, _eps) = FieldElement::sqrt_ratio_i(&eps, &one);

    let zero = FieldElement::zero();
    let Atemp = FieldElement::conditional_select(&MONTGOMERY_A, &zero, eps_is_sq); /* 0, or A if nonsquare*/
    let mut u = &d + &Atemp; /* d, or d+A if nonsquare */
    u.conditional_negate(!eps_is_sq); /* d, or -d-A if nonsquare */

    MontgomeryPoint(u.to_bytes())
}

/// A `ProjectivePoint` holds a point on the projective line
/// \\( \mathbb P(\mathbb F\_p) \\), which we identify with the Kummer
/// line of the Montgomery curve.
#[derive(Copy, Clone, Debug)]
struct ProjectivePoint {
    pub U: FieldElement,
    pub W: FieldElement,
}

impl Identity for ProjectivePoint {
    fn identity() -> ProjectivePoint {
        ProjectivePoint {
            U: FieldElement::one(),
            W: FieldElement::zero(),
        }
    }
}

impl Default for ProjectivePoint {
    fn default() -> ProjectivePoint {
        ProjectivePoint::identity()
    }
}

impl ConditionallySelectable for ProjectivePoint {
    fn conditional_select(
        a: &ProjectivePoint,
        b: &ProjectivePoint,
        choice: Choice,
    ) -> ProjectivePoint {
        ProjectivePoint {
            U: FieldElement::conditional_select(&a.U, &b.U, choice),
            W: FieldElement::conditional_select(&a.W, &b.W, choice),
        }
    }
}

impl ProjectivePoint {
    /// Dehomogenize this point to affine coordinates.
    ///
    /// # Return
    ///
    /// * \\( u = U / W \\) if \\( W \neq 0 \\);
    /// * \\( 0 \\) if \\( W \eq 0 \\);
    pub fn to_affine(&self) -> MontgomeryPoint {
        let u = &self.U * &self.W.invert();
        MontgomeryPoint(u.to_bytes())
    }
}

/// Perform the double-and-add step of the Montgomery ladder.
///
/// Given projective points
/// \\( (U\_P : W\_P) = u(P) \\),
/// \\( (U\_Q : W\_Q) = u(Q) \\),
/// and the affine difference
/// \\(      u\_{P-Q} = u(P-Q) \\), set
/// $$
///     (U\_P : W\_P) \gets u([2]P)
/// $$
/// and
/// $$
///     (U\_Q : W\_Q) \gets u(P + Q).
/// $$
fn differential_add_and_double(
    P: &mut ProjectivePoint,
    Q: &mut ProjectivePoint,
    affine_PmQ: &FieldElement,
) {
    let t0 = &P.U + &P.W;
    let t1 = &P.U - &P.W;
    let t2 = &Q.U + &Q.W;
    let t3 = &Q.U - &Q.W;

    let t4 = t0.square();   // (U_P + W_P)^2 = U_P^2 + 2 U_P W_P + W_P^2
    let t5 = t1.square();   // (U_P - W_P)^2 = U_P^2 - 2 U_P W_P + W_P^2

    let t6 = &t4 - &t5;     // 4 U_P W_P

    let t7 = &t0 * &t3;     // (U_P + W_P) (U_Q - W_Q) = U_P U_Q + W_P U_Q - U_P W_Q - W_P W_Q
    let t8 = &t1 * &t2;     // (U_P - W_P) (U_Q + W_Q) = U_P U_Q - W_P U_Q + U_P W_Q - W_P W_Q

    let t9  = &t7 + &t8;    // 2 (U_P U_Q - W_P W_Q)
    let t10 = &t7 - &t8;    // 2 (W_P U_Q - U_P W_Q)

    let t11 =  t9.square(); // 4 (U_P U_Q - W_P W_Q)^2
    let t12 = t10.square(); // 4 (W_P U_Q - U_P W_Q)^2

    let t13 = &APLUS2_OVER_FOUR * &t6; // (A + 2) U_P U_Q

    let t14 = &t4 * &t5;    // ((U_P + W_P)(U_P - W_P))^2 = (U_P^2 - W_P^2)^2
    let t15 = &t13 + &t5;   // (U_P - W_P)^2 + (A + 2) U_P W_P

    let t16 = &t6 * &t15;   // 4 (U_P W_P) ((U_P - W_P)^2 + (A + 2) U_P W_P)

    let t17 = affine_PmQ * &t12; // U_D * 4 (W_P U_Q - U_P W_Q)^2
    let t18 = t11;               // W_D * 4 (U_P U_Q - W_P W_Q)^2

    P.U = t14;  // U_{P'} = (U_P + W_P)^2 (U_P - W_P)^2
    P.W = t16;  // W_{P'} = (4 U_P W_P) ((U_P - W_P)^2 + ((A + 2)/4) 4 U_P W_P)
    Q.U = t18;  // U_{Q'} = W_D * 4 (U_P U_Q - W_P W_Q)^2
    Q.W = t17;  // W_{Q'} = U_D * 4 (W_P U_Q - U_P W_Q)^2
}

define_mul_assign_variants!(LHS = MontgomeryPoint, RHS = Scalar);

define_mul_variants!(LHS = MontgomeryPoint, RHS = Scalar, Output = MontgomeryPoint);
define_mul_variants!(LHS = Scalar, RHS = MontgomeryPoint, Output = MontgomeryPoint);

/// Multiply this `MontgomeryPoint` by a `Scalar`.
impl<'a, 'b> Mul<&'b Scalar> for &'a MontgomeryPoint {
    type Output = MontgomeryPoint;

    /// Given `self` \\( = u\_0(P) \\), and a `Scalar` \\(n\\), return \\( u\_0([n]P) \\).
    fn mul(self, scalar: &'b Scalar) -> MontgomeryPoint {
        // Algorithm 8 of Costello-Smith 2017
        let affine_u = FieldElement::from_bytes(&self.0);
        let mut x0 = ProjectivePoint::identity();
        let mut x1 = ProjectivePoint {
            U: affine_u,
            W: FieldElement::one(),
        };

        let bits: [i8; 256] = scalar.bits();

        for i in (0..255).rev() {
            let choice: u8 = (bits[i + 1] ^ bits[i]) as u8;

            debug_assert!(choice == 0 || choice == 1);

            ProjectivePoint::conditional_swap(&mut x0, &mut x1, choice.into());
            differential_add_and_double(&mut x0, &mut x1, &affine_u);
        }
        ProjectivePoint::conditional_swap(&mut x0, &mut x1, Choice::from(bits[0] as u8));

        x0.to_affine()
    }
}

impl<'b> MulAssign<&'b Scalar> for MontgomeryPoint {
    fn mul_assign(&mut self, scalar: &'b Scalar) {
        *self = (self as &MontgomeryPoint) * scalar;
    }
}

impl<'a, 'b> Mul<&'b MontgomeryPoint> for &'a Scalar {
    type Output = MontgomeryPoint;

    fn mul(self, point: &'b MontgomeryPoint) -> MontgomeryPoint {
        point * self
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use constants;
    use core::convert::TryInto;

    use rand_core::OsRng;

    #[test]
    fn identity_in_different_coordinates() {
        let id_projective = ProjectivePoint::identity();
        let id_montgomery = id_projective.to_affine();

        assert!(id_montgomery == MontgomeryPoint::identity());
    }

    #[test]
    fn identity_in_different_models() {
        assert!(EdwardsPoint::identity().to_montgomery() == MontgomeryPoint::identity());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_bincode_basepoint_roundtrip() {
        use bincode;

        let encoded = bincode::serialize(&constants::X25519_BASEPOINT).unwrap();
        let decoded: MontgomeryPoint = bincode::deserialize(&encoded).unwrap();

        assert_eq!(encoded.len(), 32);
        assert_eq!(decoded, constants::X25519_BASEPOINT);

        let raw_bytes = constants::X25519_BASEPOINT.as_bytes();
        let bp: MontgomeryPoint = bincode::deserialize(raw_bytes).unwrap();
        assert_eq!(bp, constants::X25519_BASEPOINT);
    }

    /// Test Montgomery -> Edwards on the X/Ed25519 basepoint
    #[test]
    fn basepoint_montgomery_to_edwards() {
        // sign bit = 0 => basepoint
        assert_eq!(
            constants::ED25519_BASEPOINT_POINT,
            constants::X25519_BASEPOINT.to_edwards(0).unwrap()
        );
        // sign bit = 1 => minus basepoint
        assert_eq!(
            - constants::ED25519_BASEPOINT_POINT,
            constants::X25519_BASEPOINT.to_edwards(1).unwrap()
        );
    }

    /// Test Edwards -> Montgomery on the X/Ed25519 basepoint
    #[test]
    fn basepoint_edwards_to_montgomery() {
        assert_eq!(
            constants::ED25519_BASEPOINT_POINT.to_montgomery(),
            constants::X25519_BASEPOINT
        );
    }

    /// Check that Montgomery -> Edwards fails for points on the twist.
    #[test]
    fn montgomery_to_edwards_rejects_twist() {
        let one = FieldElement::one();

        // u = 2 corresponds to a point on the twist.
        let two = MontgomeryPoint((&one+&one).to_bytes());

        assert!(two.to_edwards(0).is_none());

        // u = -1 corresponds to a point on the twist, but should be
        // checked explicitly because it's an exceptional point for the
        // birational map.  For instance, libsignal will accept it.
        let minus_one = MontgomeryPoint((-&one).to_bytes());

        assert!(minus_one.to_edwards(0).is_none());
    }

    #[test]
    fn eq_defined_mod_p() {
        let mut u18_bytes = [0u8; 32]; u18_bytes[0] = 18;
        let u18 = MontgomeryPoint(u18_bytes);
        let u18_unred = MontgomeryPoint([255; 32]);

        assert_eq!(u18, u18_unred);
    }

    #[test]
    fn montgomery_ladder_matches_edwards_scalarmult() {
        let mut csprng: OsRng = OsRng;

        let s: Scalar = Scalar::random(&mut csprng);
        let p_edwards: EdwardsPoint = &constants::ED25519_BASEPOINT_TABLE * &s;
        let p_montgomery: MontgomeryPoint = p_edwards.to_montgomery();

        let expected = s * p_edwards;
        let result = s * p_montgomery;

        assert_eq!(result, expected.to_montgomery())
    }

    const ELLIGATOR_CORRECT_OUTPUT: [u8; 32] = [
        0x5f, 0x35, 0x20, 0x00, 0x1c, 0x6c, 0x99, 0x36, 0xa3, 0x12, 0x06, 0xaf, 0xe7, 0xc7, 0xac,
        0x22, 0x4e, 0x88, 0x61, 0x61, 0x9b, 0xf9, 0x88, 0x72, 0x44, 0x49, 0x15, 0x89, 0x9d, 0x95,
        0xf4, 0x6e,
    ];

    #[test]
    #[cfg(feature = "std")] // Vec
    fn montgomery_elligator_correct() {
        let bytes: std::vec::Vec<u8> = (0u8..32u8).collect();
        let bits_in: [u8; 32] = (&bytes[..]).try_into().expect("Range invariant broken");

        let fe = FieldElement::from_bytes(&bits_in);
        let eg = elligator_encode(&fe);
        assert_eq!(eg.to_bytes(), ELLIGATOR_CORRECT_OUTPUT);
    }

    #[test]
    fn montgomery_elligator_zero_zero() {
        let zero = [0u8; 32];
        let fe = FieldElement::from_bytes(&zero);
        let eg = elligator_encode(&fe);
        assert_eq!(eg.to_bytes(), zero);
    }
}
