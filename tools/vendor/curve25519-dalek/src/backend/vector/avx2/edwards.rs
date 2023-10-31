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

//! Parallel Edwards Arithmetic for Curve25519.
//!
//! This module currently has two point types:
//!
//! * `ExtendedPoint`: a point stored in vector-friendly format, with
//! vectorized doubling and addition;
//!
//! * `CachedPoint`: used for readdition.
//!
//! Details on the formulas can be found in the documentation for the
//! parent `avx2` module.
//!
//! This API is designed to be safe: vectorized points can only be
//! created from serial points (which do validation on decompression),
//! and operations on valid points return valid points, so invalid
//! point states should be unrepresentable.
//!
//! This design goal is met, with one exception: the `Neg`
//! implementation for the `CachedPoint` performs a lazy negation, so
//! that subtraction can be efficiently implemented as a negation and
//! an addition.  Repeatedly negating a `CachedPoint` will cause its
//! coefficients to grow and eventually overflow.  Repeatedly negating
//! a point should not be necessary anyways.

#![allow(non_snake_case)]

use core::convert::From;
use core::ops::{Add, Neg, Sub};

use subtle::Choice;
use subtle::ConditionallySelectable;

use edwards;
use window::{LookupTable, NafLookupTable5, NafLookupTable8};

use traits::Identity;

use super::constants;
use super::field::{FieldElement2625x4, Lanes, Shuffle};

/// A point on Curve25519, using parallel Edwards formulas for curve
/// operations.
///
/// # Invariant
///
/// The coefficients of an `ExtendedPoint` are bounded with
/// \\( b < 0.007 \\).
#[derive(Copy, Clone, Debug)]
pub struct ExtendedPoint(pub(super) FieldElement2625x4);

impl From<edwards::EdwardsPoint> for ExtendedPoint {
    fn from(P: edwards::EdwardsPoint) -> ExtendedPoint {
        ExtendedPoint(FieldElement2625x4::new(&P.X, &P.Y, &P.Z, &P.T))
    }
}

impl From<ExtendedPoint> for edwards::EdwardsPoint {
    fn from(P: ExtendedPoint) -> edwards::EdwardsPoint {
        let tmp = P.0.split();
        edwards::EdwardsPoint {
            X: tmp[0],
            Y: tmp[1],
            Z: tmp[2],
            T: tmp[3],
        }
    }
}

impl ConditionallySelectable for ExtendedPoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        ExtendedPoint(FieldElement2625x4::conditional_select(&a.0, &b.0, choice))
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.0.conditional_assign(&other.0, choice);
    }
}

impl Default for ExtendedPoint {
    fn default() -> ExtendedPoint {
        ExtendedPoint::identity()
    }
}

impl Identity for ExtendedPoint {
    fn identity() -> ExtendedPoint {
        constants::EXTENDEDPOINT_IDENTITY
    }
}

impl ExtendedPoint {
    /// Compute the double of this point.
    pub fn double(&self) -> ExtendedPoint {
        // Want to compute (X1 Y1 Z1 X1+Y1).
        // Not sure how to do this less expensively than computing
        // (X1 Y1 Z1 T1) --(256bit shuffle)--> (X1 Y1 X1 Y1)
        // (X1 Y1 X1 Y1) --(2x128b shuffle)--> (Y1 X1 Y1 X1)
        // and then adding.

        // Set tmp0 = (X1 Y1 X1 Y1)
        let mut tmp0 = self.0.shuffle(Shuffle::ABAB);

        // Set tmp1 = (Y1 X1 Y1 X1)
        let mut tmp1 = tmp0.shuffle(Shuffle::BADC);

        // Set tmp0 = (X1 Y1 Z1 X1+Y1)
        tmp0 = self.0.blend(tmp0 + tmp1, Lanes::D);

        // Set tmp1 = tmp0^2, negating the D values
        tmp1 = tmp0.square_and_negate_D();
        // Now tmp1 = (S1 S2 S3 -S4) with b < 0.007

        // See discussion of bounds in the module-level documentation.
        // We want to compute
        //
        //    + | S1 | S1 | S1 | S1 |
        //    + | S2 |    |    | S2 |
        //    + |    |    | S3 |    |
        //    + |    |    | S3 |    |
        //    + |    |    |    |-S4 |
        //    + |    | 2p | 2p |    |
        //    - |    | S2 | S2 |    |
        //    =======================
        //        S5   S6   S8   S9

        let zero = FieldElement2625x4::zero();
        let S_1 = tmp1.shuffle(Shuffle::AAAA);
        let S_2 = tmp1.shuffle(Shuffle::BBBB);

        tmp0 = zero.blend(tmp1 + tmp1, Lanes::C);
        // tmp0 = (0, 0,  2S_3, 0)
        tmp0 = tmp0.blend(tmp1, Lanes::D);
        // tmp0 = (0, 0,  2S_3, -S_4)
        tmp0 = tmp0 + S_1;
        // tmp0 = (  S_1,   S_1, S_1 + 2S_3, S_1 - S_4)
        tmp0 = tmp0 + zero.blend(S_2, Lanes::AD);
        // tmp0 = (S_1 + S_2,   S_1, S_1 + 2S_3, S_1 + S_2 - S_4)
        tmp0 = tmp0 + zero.blend(S_2.negate_lazy(), Lanes::BC);
        // tmp0 = (S_1 + S_2, S_1 - S_2, S_1 - S_2 + 2S_3, S_1 + S_2 - S_4)
        //    b < (     1.01,       1.6,             2.33,             1.6)
        // Now tmp0 = (S_5, S_6, S_8, S_9)

        // Set tmp1 = ( S_9,  S_6,  S_6,  S_9)
        //        b < ( 1.6,  1.6,  1.6,  1.6)
        tmp1 = tmp0.shuffle(Shuffle::DBBD);
        // Set tmp0 = ( S_8,  S_5,  S_8,  S_5)
        //        b < (2.33, 1.01, 2.33, 1.01)
        tmp0 = tmp0.shuffle(Shuffle::CACA);

        // Bounds on (tmp0, tmp1) are (2.33, 1.6) < (2.5, 1.75).
        ExtendedPoint(&tmp0 * &tmp1)
    }

    pub fn mul_by_pow_2(&self, k: u32) -> ExtendedPoint {
        let mut tmp: ExtendedPoint = *self;
        for _ in 0..k {
            tmp = tmp.double();
        }
        tmp
    }
}

/// A cached point with some precomputed variables used for readdition.
///
/// # Warning
///
/// It is not safe to negate this point more than once.
///
/// # Invariant
///
/// As long as the `CachedPoint` is not repeatedly negated, its
/// coefficients will be bounded with \\( b < 1.0 \\).
#[derive(Copy, Clone, Debug)]
pub struct CachedPoint(pub(super) FieldElement2625x4);

impl From<ExtendedPoint> for CachedPoint {
    fn from(P: ExtendedPoint) -> CachedPoint {
        let mut x = P.0;

        x = x.blend(x.diff_sum(), Lanes::AB);
        // x = (Y2 - X2, Y2 + X2, Z2, T2) = (S2 S3 Z2 T2)

        x = x * (121666, 121666, 2 * 121666, 2 * 121665);
        // x = (121666*S2 121666*S3 2*121666*Z2 2*121665*T2)

        x = x.blend(-x, Lanes::D);
        // x = (121666*S2 121666*S3 2*121666*Z2 -2*121665*T2)

        // The coefficients of the output are bounded with b < 0.007.
        CachedPoint(x)
    }
}

impl Default for CachedPoint {
    fn default() -> CachedPoint {
        CachedPoint::identity()
    }
}

impl Identity for CachedPoint {
    fn identity() -> CachedPoint {
        constants::CACHEDPOINT_IDENTITY
    }
}

impl ConditionallySelectable for CachedPoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        CachedPoint(FieldElement2625x4::conditional_select(&a.0, &b.0, choice))
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.0.conditional_assign(&other.0, choice);
    }
}

impl<'a> Neg for &'a CachedPoint {
    type Output = CachedPoint;
    /// Lazily negate the point.
    ///
    /// # Warning
    ///
    /// Because this method does not perform a reduction, it is not
    /// safe to repeatedly negate a point.
    fn neg(self) -> CachedPoint {
        let swapped = self.0.shuffle(Shuffle::BACD);
        CachedPoint(swapped.blend(swapped.negate_lazy(), Lanes::D))
    }
}

impl<'a, 'b> Add<&'b CachedPoint> for &'a ExtendedPoint {
    type Output = ExtendedPoint;

    /// Add an `ExtendedPoint` and a `CachedPoint`.
    fn add(self, other: &'b CachedPoint) -> ExtendedPoint {
        // The coefficients of an `ExtendedPoint` are reduced after
        // every operation.  If the `CachedPoint` was negated, its
        // coefficients grow by one bit.  So on input, `self` is
        // bounded with `b < 0.007` and `other` is bounded with
        // `b < 1.0`.

        let mut tmp = self.0;

        tmp = tmp.blend(tmp.diff_sum(), Lanes::AB);
        // tmp = (Y1-X1 Y1+X1 Z1 T1) = (S0 S1 Z1 T1) with b < 1.6

        // (tmp, other) bounded with b < (1.6, 1.0) < (2.5, 1.75).
        tmp = &tmp * &other.0;
        // tmp = (S0*S2' S1*S3' Z1*Z2' T1*T2') = (S8 S9 S10 S11)

        tmp = tmp.shuffle(Shuffle::ABDC);
        // tmp = (S8 S9 S11 S10)

        tmp = tmp.diff_sum();
        // tmp = (S9-S8 S9+S8 S10-S11 S10+S11) = (S12 S13 S14 S15)

        let t0 = tmp.shuffle(Shuffle::ADDA);
        // t0 = (S12 S15 S15 S12)
        let t1 = tmp.shuffle(Shuffle::CBCB);
        // t1 = (S14 S13 S14 S13)

        // All coefficients of t0, t1 are bounded with b < 1.6.
        // Return (S12*S14 S15*S13 S15*S14 S12*S13) = (X3 Y3 Z3 T3)
        ExtendedPoint(&t0 * &t1)
    }
}

impl<'a, 'b> Sub<&'b CachedPoint> for &'a ExtendedPoint {
    type Output = ExtendedPoint;

    /// Implement subtraction by negating the point and adding.
    ///
    /// Empirically, this seems about the same cost as a custom
    /// subtraction impl (maybe because the benefit is cancelled by
    /// increased code size?)
    fn sub(self, other: &'b CachedPoint) -> ExtendedPoint {
        self + &(-other)
    }
}

impl<'a> From<&'a edwards::EdwardsPoint> for LookupTable<CachedPoint> {
    fn from(point: &'a edwards::EdwardsPoint) -> Self {
        let P = ExtendedPoint::from(*point);
        let mut points = [CachedPoint::from(P); 8];
        for i in 0..7 {
            points[i + 1] = (&P + &points[i]).into();
        }
        LookupTable(points)
    }
}

impl<'a> From<&'a edwards::EdwardsPoint> for NafLookupTable5<CachedPoint> {
    fn from(point: &'a edwards::EdwardsPoint) -> Self {
        let A = ExtendedPoint::from(*point);
        let mut Ai = [CachedPoint::from(A); 8];
        let A2 = A.double();
        for i in 0..7 {
            Ai[i + 1] = (&A2 + &Ai[i]).into();
        }
        // Now Ai = [A, 3A, 5A, 7A, 9A, 11A, 13A, 15A]
        NafLookupTable5(Ai)
    }
}

impl<'a> From<&'a edwards::EdwardsPoint> for NafLookupTable8<CachedPoint> {
    fn from(point: &'a edwards::EdwardsPoint) -> Self {
        let A = ExtendedPoint::from(*point);
        let mut Ai = [CachedPoint::from(A); 64];
        let A2 = A.double();
        for i in 0..63 {
            Ai[i + 1] = (&A2 + &Ai[i]).into();
        }
        // Now Ai = [A, 3A, 5A, 7A, 9A, 11A, 13A, 15A, ..., 127A]
        NafLookupTable8(Ai)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn serial_add(P: edwards::EdwardsPoint, Q: edwards::EdwardsPoint) -> edwards::EdwardsPoint {
        use backend::serial::u64::field::FieldElement51;

        let (X1, Y1, Z1, T1) = (P.X, P.Y, P.Z, P.T);
        let (X2, Y2, Z2, T2) = (Q.X, Q.Y, Q.Z, Q.T);

        macro_rules! print_var {
            ($x:ident) => {
                println!("{} = {:?}", stringify!($x), $x.to_bytes());
            };
        }

        let S0 = &Y1 - &X1; // R1
        let S1 = &Y1 + &X1; // R3
        let S2 = &Y2 - &X2; // R2
        let S3 = &Y2 + &X2; // R4
        print_var!(S0);
        print_var!(S1);
        print_var!(S2);
        print_var!(S3);
        println!("");

        let S4 = &S0 * &S2; // R5 = R1 * R2
        let S5 = &S1 * &S3; // R6 = R3 * R4
        let S6 = &Z1 * &Z2; // R8
        let S7 = &T1 * &T2; // R7
        print_var!(S4);
        print_var!(S5);
        print_var!(S6);
        print_var!(S7);
        println!("");

        let S8  =  &S4 *    &FieldElement51([  121666,0,0,0,0]);  // R5
        let S9  =  &S5 *    &FieldElement51([  121666,0,0,0,0]);  // R6
        let S10 =  &S6 *    &FieldElement51([2*121666,0,0,0,0]);  // R8
        let S11 =  &S7 * &(-&FieldElement51([2*121665,0,0,0,0])); // R7
        print_var!(S8);
        print_var!(S9);
        print_var!(S10);
        print_var!(S11);
        println!("");

        let S12 =  &S9 - &S8;  // R1
        let S13 =  &S9 + &S8;  // R4
        let S14 = &S10 - &S11; // R2
        let S15 = &S10 + &S11; // R3
        print_var!(S12);
        print_var!(S13);
        print_var!(S14);
        print_var!(S15);
        println!("");

        let X3 = &S12 * &S14; // R1 * R2
        let Y3 = &S15 * &S13; // R3 * R4
        let Z3 = &S15 * &S14; // R2 * R3
        let T3 = &S12 * &S13; // R1 * R4

        edwards::EdwardsPoint {
            X: X3,
            Y: Y3,
            Z: Z3,
            T: T3,
        }
    }

    fn addition_test_helper(P: edwards::EdwardsPoint, Q: edwards::EdwardsPoint) {
        // Test the serial implementation of the parallel addition formulas
        let R_serial: edwards::EdwardsPoint = serial_add(P.into(), Q.into()).into();

        // Test the vector implementation of the parallel readdition formulas
        let cached_Q = CachedPoint::from(ExtendedPoint::from(Q));
        let R_vector: edwards::EdwardsPoint = (&ExtendedPoint::from(P) + &cached_Q).into();
        let S_vector: edwards::EdwardsPoint = (&ExtendedPoint::from(P) - &cached_Q).into();

        println!("Testing point addition:");
        println!("P = {:?}", P);
        println!("Q = {:?}", Q);
        println!("cached Q = {:?}", cached_Q);
        println!("R = P + Q = {:?}", &P + &Q);
        println!("R_serial = {:?}", R_serial);
        println!("R_vector = {:?}", R_vector);
        println!("S = P - Q = {:?}", &P - &Q);
        println!("S_vector = {:?}", S_vector);
        assert_eq!(R_serial.compress(), (&P + &Q).compress());
        assert_eq!(R_vector.compress(), (&P + &Q).compress());
        assert_eq!(S_vector.compress(), (&P - &Q).compress());
        println!("OK!\n");
    }

    #[test]
    fn vector_addition_vs_serial_addition_vs_edwards_extendedpoint() {
        use constants;
        use scalar::Scalar;

        println!("Testing id +- id");
        let P = edwards::EdwardsPoint::identity();
        let Q = edwards::EdwardsPoint::identity();
        addition_test_helper(P, Q);

        println!("Testing id +- B");
        let P = edwards::EdwardsPoint::identity();
        let Q = constants::ED25519_BASEPOINT_POINT;
        addition_test_helper(P, Q);

        println!("Testing B +- B");
        let P = constants::ED25519_BASEPOINT_POINT;
        let Q = constants::ED25519_BASEPOINT_POINT;
        addition_test_helper(P, Q);

        println!("Testing B +- kB");
        let P = constants::ED25519_BASEPOINT_POINT;
        let Q = &constants::ED25519_BASEPOINT_TABLE * &Scalar::from(8475983829u64);
        addition_test_helper(P, Q);
    }

    fn serial_double(P: edwards::EdwardsPoint) -> edwards::EdwardsPoint {
        let (X1, Y1, Z1, _T1) = (P.X, P.Y, P.Z, P.T);

        macro_rules! print_var {
            ($x:ident) => {
                println!("{} = {:?}", stringify!($x), $x.to_bytes());
            };
        }

        let S0 = &X1 + &Y1; // R1
        print_var!(S0);
        println!("");

        let S1 = X1.square();
        let S2 = Y1.square();
        let S3 = Z1.square();
        let S4 = S0.square();
        print_var!(S1);
        print_var!(S2);
        print_var!(S3);
        print_var!(S4);
        println!("");

        let S5 = &S1 + &S2;
        let S6 = &S1 - &S2;
        let S7 = &S3 + &S3;
        let S8 = &S7 + &S6;
        let S9 = &S5 - &S4;
        print_var!(S5);
        print_var!(S6);
        print_var!(S7);
        print_var!(S8);
        print_var!(S9);
        println!("");

        let X3 = &S8 * &S9;
        let Y3 = &S5 * &S6;
        let Z3 = &S8 * &S6;
        let T3 = &S5 * &S9;

        edwards::EdwardsPoint {
            X: X3,
            Y: Y3,
            Z: Z3,
            T: T3,
        }
    }

    fn doubling_test_helper(P: edwards::EdwardsPoint) {
        let R1: edwards::EdwardsPoint = serial_double(P.into()).into();
        let R2: edwards::EdwardsPoint = ExtendedPoint::from(P).double().into();
        println!("Testing point doubling:");
        println!("P = {:?}", P);
        println!("(serial) R1 = {:?}", R1);
        println!("(vector) R2 = {:?}", R2);
        println!("P + P = {:?}", &P + &P);
        assert_eq!(R1.compress(), (&P + &P).compress());
        assert_eq!(R2.compress(), (&P + &P).compress());
        println!("OK!\n");
    }

    #[test]
    fn vector_doubling_vs_serial_doubling_vs_edwards_extendedpoint() {
        use constants;
        use scalar::Scalar;

        println!("Testing [2]id");
        let P = edwards::EdwardsPoint::identity();
        doubling_test_helper(P);

        println!("Testing [2]B");
        let P = constants::ED25519_BASEPOINT_POINT;
        doubling_test_helper(P);

        println!("Testing [2]([k]B)");
        let P = &constants::ED25519_BASEPOINT_TABLE * &Scalar::from(8475983829u64);
        doubling_test_helper(P);
    }

    #[test]
    fn basepoint_odd_lookup_table_verify() {
        use constants;
        use backend::vector::avx2::constants::{BASEPOINT_ODD_LOOKUP_TABLE};

        let basepoint_odd_table = NafLookupTable8::<CachedPoint>::from(&constants::ED25519_BASEPOINT_POINT);
        println!("basepoint_odd_lookup_table = {:?}", basepoint_odd_table);

        let table_B = &BASEPOINT_ODD_LOOKUP_TABLE;
        for (b_vec, base_vec) in table_B.0.iter().zip(basepoint_odd_table.0.iter()) {
            let b_splits = b_vec.0.split();
            let base_splits = base_vec.0.split();

            assert_eq!(base_splits[0], b_splits[0]);
            assert_eq!(base_splits[1], b_splits[1]);
            assert_eq!(base_splits[2], b_splits[2]);
            assert_eq!(base_splits[3], b_splits[3]);
        }
    }
}
