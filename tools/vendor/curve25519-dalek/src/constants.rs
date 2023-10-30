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

//! Various constants, such as the Ristretto and Ed25519 basepoints.
//!
//! Most of the constants are given with
//! `LONG_DESCRIPTIVE_UPPER_CASE_NAMES`, but they can be brought into
//! scope using a `let` binding:
//!
//! ```
//! use curve25519_dalek::constants;
//! use curve25519_dalek::traits::IsIdentity;
//!
//! let B = &constants::RISTRETTO_BASEPOINT_TABLE;
//! let l = &constants::BASEPOINT_ORDER;
//!
//! let A = l * B;
//! assert!(A.is_identity());
//! ```

#![allow(non_snake_case)]

use edwards::CompressedEdwardsY;
use ristretto::RistrettoPoint;
use ristretto::CompressedRistretto;
use montgomery::MontgomeryPoint;
use scalar::Scalar;

#[cfg(feature = "fiat_u32_backend")]
pub use backend::serial::fiat_u32::constants::*;
#[cfg(feature = "fiat_u64_backend")]
pub use backend::serial::fiat_u64::constants::*;
#[cfg(feature = "u64_backend")]
pub use backend::serial::u64::constants::*;
#[cfg(feature = "u32_backend")]
pub use backend::serial::u32::constants::*;

/// The Ed25519 basepoint, in `CompressedEdwardsY` format.
///
/// This is the little-endian byte encoding of \\( 4/5 \pmod p \\),
/// which is the \\(y\\)-coordinate of the Ed25519 basepoint.
///
/// The sign bit is 0 since the basepoint has \\(x\\) chosen to be positive.
pub const ED25519_BASEPOINT_COMPRESSED: CompressedEdwardsY =
    CompressedEdwardsY([0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
                        0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
                        0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
                        0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66]);

/// The X25519 basepoint, in `MontgomeryPoint` format.
pub const X25519_BASEPOINT: MontgomeryPoint =
    MontgomeryPoint([0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

/// The Ristretto basepoint, in `CompressedRistretto` format.
pub const RISTRETTO_BASEPOINT_COMPRESSED: CompressedRistretto =
    CompressedRistretto([0xe2, 0xf2, 0xae, 0x0a, 0x6a, 0xbc, 0x4e, 0x71,
                         0xa8, 0x84, 0xa9, 0x61, 0xc5, 0x00, 0x51, 0x5f,
                         0x58, 0xe3, 0x0b, 0x6a, 0xa5, 0x82, 0xdd, 0x8d,
                         0xb6, 0xa6, 0x59, 0x45, 0xe0, 0x8d, 0x2d, 0x76]);

/// The Ristretto basepoint, as a `RistrettoPoint`.
///
/// This is called `_POINT` to distinguish it from `_TABLE`, which
/// provides fast scalar multiplication.
pub const RISTRETTO_BASEPOINT_POINT: RistrettoPoint = RistrettoPoint(ED25519_BASEPOINT_POINT);

/// `BASEPOINT_ORDER` is the order of the Ristretto group and of the Ed25519 basepoint, i.e.,
/// $$
/// \ell = 2^\{252\} + 27742317777372353535851937790883648493.
/// $$
pub const BASEPOINT_ORDER: Scalar = Scalar{
    bytes: [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
        0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ],
};

use ristretto::RistrettoBasepointTable;
/// The Ristretto basepoint, as a `RistrettoBasepointTable` for scalar multiplication.
pub const RISTRETTO_BASEPOINT_TABLE: RistrettoBasepointTable
    = RistrettoBasepointTable(ED25519_BASEPOINT_TABLE);

#[cfg(test)]
mod test {
    use field::FieldElement;
    use traits::{IsIdentity, ValidityCheck};
    use constants;

    #[test]
    fn test_eight_torsion() {
        for i in 0..8 {
            let Q = constants::EIGHT_TORSION[i].mul_by_pow_2(3);
            assert!(Q.is_valid());
            assert!(Q.is_identity());
        }
    }

    #[test]
    fn test_four_torsion() {
        for i in (0..8).filter(|i| i % 2 == 0) {
            let Q = constants::EIGHT_TORSION[i].mul_by_pow_2(2);
            assert!(Q.is_valid());
            assert!(Q.is_identity());
        }
    }

    #[test]
    fn test_two_torsion() {
        for i in (0..8).filter(|i| i % 4 == 0) {
            let Q = constants::EIGHT_TORSION[i].mul_by_pow_2(1);
            assert!(Q.is_valid());
            assert!(Q.is_identity());
        }
    }

    /// Test that SQRT_M1 is the positive square root of -1
    #[test]
    fn test_sqrt_minus_one() {
        let minus_one = FieldElement::minus_one();
        let sqrt_m1_sq = &constants::SQRT_M1 * &constants::SQRT_M1;
        assert_eq!(minus_one,  sqrt_m1_sq);
        assert_eq!(constants::SQRT_M1.is_negative().unwrap_u8(), 0);
    }

    #[test]
    fn test_sqrt_constants_sign() {
        let minus_one = FieldElement::minus_one();
        let (was_nonzero_square, invsqrt_m1) = minus_one.invsqrt();
        assert_eq!(was_nonzero_square.unwrap_u8(), 1u8);
        let sign_test_sqrt  = &invsqrt_m1 * &constants::SQRT_M1;
        assert_eq!(sign_test_sqrt, minus_one);
    }

    /// Test that d = -121665/121666
    #[test]
    #[cfg(feature = "u32_backend")]
    fn test_d_vs_ratio() {
        use backend::serial::u32::field::FieldElement2625;
        let a = -&FieldElement2625([121665,0,0,0,0,0,0,0,0,0]);
        let b =   FieldElement2625([121666,0,0,0,0,0,0,0,0,0]);
        let d = &a * &b.invert();
        let d2 = &d + &d;
        assert_eq!(d, constants::EDWARDS_D);
        assert_eq!(d2, constants::EDWARDS_D2);
    }

    /// Test that d = -121665/121666
    #[test]
    #[cfg(feature = "u64_backend")]
    fn test_d_vs_ratio() {
        use backend::serial::u64::field::FieldElement51;
        let a = -&FieldElement51([121665,0,0,0,0]);
        let b =   FieldElement51([121666,0,0,0,0]);
        let d = &a * &b.invert();
        let d2 = &d + &d;
        assert_eq!(d, constants::EDWARDS_D);
        assert_eq!(d2, constants::EDWARDS_D2);
    }

    #[test]
    fn test_sqrt_ad_minus_one() {
        let a = FieldElement::minus_one();
        let ad_minus_one = &(&a * &constants::EDWARDS_D) + &a;
        let should_be_ad_minus_one = constants::SQRT_AD_MINUS_ONE.square();
        assert_eq!(should_be_ad_minus_one, ad_minus_one);
    }

}
