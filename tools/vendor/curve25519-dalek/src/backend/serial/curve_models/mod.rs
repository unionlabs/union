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

//! Internal curve representations which are not part of the public API.
//!
//! # Curve representations
//!
//! Internally, we use several different models for the curve.  Here
//! is a sketch of the relationship between the models, following [a
//! post][smith-moderncrypto]
//! by Ben Smith on the `moderncrypto` mailing list.  This is also briefly
//! discussed in section 2.5 of [_Montgomery curves and their
//! arithmetic_][costello-smith-2017] by Costello and Smith.
//!
//! Begin with the affine equation for the curve,
//! $$
//!     -x\^2 + y\^2 = 1 + dx\^2y\^2.
//! $$
//! Next, pass to the projective closure \\(\mathbb P\^1 \times \mathbb
//! P\^1 \\) by setting \\(x=X/Z\\), \\(y=Y/T.\\)  Clearing denominators
//! gives the model
//! $$
//!     -X\^2T\^2 + Y\^2Z\^2 = Z\^2T\^2 + dX\^2Y\^2.
//! $$
//! In `curve25519-dalek`, this is represented as the `CompletedPoint`
//! struct.
//! To map from \\(\mathbb P\^1 \times \mathbb P\^1 \\), a product of
//! two lines, to \\(\mathbb P\^3\\), we use the [Segre
//! embedding](https://en.wikipedia.org/wiki/Segre_embedding)
//! $$
//!     \sigma : ((X:Z),(Y:T)) \mapsto (XY:XT:ZY:ZT).
//! $$
//! Using coordinates \\( (W_0:W_1:W_2:W_3) \\) for \\(\mathbb P\^3\\),
//! the image \\(\sigma (\mathbb P\^1 \times \mathbb P\^1) \\) is the
//! surface defined by \\( W_0 W_3 = W_1 W_2 \\), and under \\(
//! \sigma\\), the equation above becomes
//! $$
//!     -W\_1\^2 + W\_2\^2 = W\_3\^2 + dW\_0\^2,
//! $$
//! so that the curve is given by the pair of equations
//! $$
//! \begin{aligned}
//!     -W\_1\^2 + W\_2\^2 &= W\_3\^2 + dW\_0\^2, \\\\  W_0 W_3 &= W_1 W_2.
//! \end{aligned}
//! $$
//! Up to variable naming, this is exactly the "extended" curve model
//! introduced in [_Twisted Edwards Curves
//! Revisited_][hisil-wong-carter-dawson-2008] by Hisil, Wong, Carter,
//! and Dawson.  In `curve25519-dalek`, it is represented as the
//! `EdwardsPoint` struct.  We can map from \\(\mathbb P\^3 \\) to
//! \\(\mathbb P\^2 \\) by sending \\( (W\_0:W\_1:W\_2:W\_3) \\) to \\(
//! (W\_1:W\_2:W\_3) \\).  Notice that
//! $$
//!     \frac {W\_1} {W\_3} = \frac {XT} {ZT} = \frac X Z = x,
//! $$
//! and
//! $$
//!     \frac {W\_2} {W\_3} = \frac {YZ} {ZT} = \frac Y T = y,
//! $$
//! so this is the same as if we had started with the affine model
//! and passed to \\( \mathbb P\^2 \\) by setting \\( x = W\_1 / W\_3
//! \\), \\(y = W\_2 / W\_3 \\).
//! Up to variable naming, this is the projective representation
//! introduced in in [_Twisted Edwards
//! Curves_][bernstein-birkner-joye-lange-peters-2008] by Bernstein,
//! Birkner, Joye, Lange, and Peters.  In `curve25519-dalek`, it is
//! represented by the `ProjectivePoint` struct.
//!
//! # Passing between curve models
//!
//! Although the \\( \mathbb P\^3 \\) model provides faster addition
//! formulas, the \\( \mathbb P\^2 \\) model provides faster doubling
//! formulas.  Hisil, Wong, Carter, and Dawson therefore suggest mixing
//! coordinate systems for scalar multiplication, attributing the idea
//! to [a 1998 paper][cohen-miyaji-ono-1998] of Cohen, Miyagi, and Ono.
//!
//! Their suggestion is to vary the formulas used by context, using a
//! \\( \mathbb P\^2 \rightarrow \mathbb P\^2 \\) doubling formula when
//! a doubling is followed
//! by another doubling, a \\( \mathbb P\^2 \rightarrow \mathbb P\^3 \\)
//! doubling formula when a doubling is followed by an addition, and
//! computing point additions using a \\( \mathbb P\^3 \times \mathbb P\^3
//! \rightarrow \mathbb P\^2 \\) formula.
//!
//! The `ref10` reference implementation of [Ed25519][ed25519], by
//! Bernstein, Duif, Lange, Schwabe, and Yang, tweaks
//! this strategy, factoring the addition formulas through the
//! completion \\( \mathbb P\^1 \times \mathbb P\^1 \\), so that the
//! output of an addition or doubling always lies in \\( \mathbb P\^1 \times
//! \mathbb P\^1\\), and the choice of which formula to use is replaced
//! by a choice of whether to convert the result to \\( \mathbb P\^2 \\)
//! or \\(\mathbb P\^3 \\).  However, this tweak is not described in
//! their paper, only in their software.
//!
//! Our naming for the `CompletedPoint` (\\(\mathbb P\^1 \times \mathbb
//! P\^1 \\)), `ProjectivePoint` (\\(\mathbb P\^2 \\)), and
//! `EdwardsPoint` (\\(\mathbb P\^3 \\)) structs follows the naming in
//! Adam Langley's [Golang ed25519][agl-ed25519] implementation, which
//! `curve25519-dalek` was originally derived from.
//!
//! Finally, to accelerate readditions, we use two cached point formats
//! in "Niels coordinates", named for Niels Duif,
//! one for the affine model and one for the \\( \mathbb P\^3 \\) model:
//!
//! * `AffineNielsPoint`: \\( (y+x, y-x, 2dxy) \\)
//! * `ProjectiveNielsPoint`: \\( (Y+X, Y-X, Z, 2dXY) \\)
//!
//! [smith-moderncrypto]: https://moderncrypto.org/mail-archive/curves/2016/000807.html
//! [costello-smith-2017]: https://eprint.iacr.org/2017/212
//! [hisil-wong-carter-dawson-2008]: https://www.iacr.org/archive/asiacrypt2008/53500329/53500329.pdf
//! [bernstein-birkner-joye-lange-peters-2008]: https://eprint.iacr.org/2008/013
//! [cohen-miyaji-ono-1998]: https://link.springer.com/content/pdf/10.1007%2F3-540-49649-1_6.pdf
//! [ed25519]: https://eprint.iacr.org/2011/368
//! [agl-ed25519]: https://github.com/agl/ed25519

#![allow(non_snake_case)]

use core::fmt::Debug;
use core::ops::{Add, Neg, Sub};

use subtle::Choice;
use subtle::ConditionallySelectable;

use zeroize::Zeroize;

use constants;

use edwards::EdwardsPoint;
use field::FieldElement;
use traits::ValidityCheck;

// ------------------------------------------------------------------------
// Internal point representations
// ------------------------------------------------------------------------

/// A `ProjectivePoint` is a point \\((X:Y:Z)\\) on the \\(\mathbb
/// P\^2\\) model of the curve.
/// A point \\((x,y)\\) in the affine model corresponds to
/// \\((x:y:1)\\).
///
/// More details on the relationships between the different curve models
/// can be found in the module-level documentation.
#[derive(Copy, Clone)]
pub struct ProjectivePoint {
    pub X: FieldElement,
    pub Y: FieldElement,
    pub Z: FieldElement,
}

/// A `CompletedPoint` is a point \\(((X:Z), (Y:T))\\) on the \\(\mathbb
/// P\^1 \times \mathbb P\^1 \\) model of the curve.
/// A point (x,y) in the affine model corresponds to \\( ((x:1),(y:1))
/// \\).
///
/// More details on the relationships between the different curve models
/// can be found in the module-level documentation.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub struct CompletedPoint {
    pub X: FieldElement,
    pub Y: FieldElement,
    pub Z: FieldElement,
    pub T: FieldElement,
}

/// A pre-computed point in the affine model for the curve, represented as
/// \\((y+x, y-x, 2dxy)\\) in "Niels coordinates".
///
/// More details on the relationships between the different curve models
/// can be found in the module-level documentation.
// Safe to derive Eq because affine coordinates.
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(missing_docs)]
pub struct AffineNielsPoint {
    pub y_plus_x:  FieldElement,
    pub y_minus_x: FieldElement,
    pub xy2d:      FieldElement,
}

impl Zeroize for AffineNielsPoint {
    fn zeroize(&mut self) {
        self.y_plus_x.zeroize();
        self.y_minus_x.zeroize();
        self.xy2d.zeroize();
    }
}

/// A pre-computed point on the \\( \mathbb P\^3 \\) model for the
/// curve, represented as \\((Y+X, Y-X, Z, 2dXY)\\) in "Niels coordinates".
///
/// More details on the relationships between the different curve models
/// can be found in the module-level documentation.
#[derive(Copy, Clone)]
pub struct ProjectiveNielsPoint {
    pub Y_plus_X:  FieldElement,
    pub Y_minus_X: FieldElement,
    pub Z:         FieldElement,
    pub T2d:       FieldElement,
}

impl Zeroize for ProjectiveNielsPoint {
    fn zeroize(&mut self) {
        self.Y_plus_X.zeroize();
        self.Y_minus_X.zeroize();
        self.Z.zeroize();
        self.T2d.zeroize();
    }
}

// ------------------------------------------------------------------------
// Constructors
// ------------------------------------------------------------------------

use traits::Identity;

impl Identity for ProjectivePoint {
    fn identity() -> ProjectivePoint {
        ProjectivePoint {
            X: FieldElement::zero(),
            Y: FieldElement::one(),
            Z: FieldElement::one(),
        }
    }
}

impl Identity for ProjectiveNielsPoint {
    fn identity() -> ProjectiveNielsPoint {
        ProjectiveNielsPoint{
            Y_plus_X:  FieldElement::one(),
            Y_minus_X: FieldElement::one(),
            Z:         FieldElement::one(),
            T2d:       FieldElement::zero(),
        }
    }
}

impl Default for ProjectiveNielsPoint {
    fn default() -> ProjectiveNielsPoint {
        ProjectiveNielsPoint::identity()
    }
}

impl Identity for AffineNielsPoint {
    fn identity() -> AffineNielsPoint {
        AffineNielsPoint{
            y_plus_x:  FieldElement::one(),
            y_minus_x: FieldElement::one(),
            xy2d:      FieldElement::zero(),
        }
    }
}

impl Default for AffineNielsPoint {
    fn default() -> AffineNielsPoint {
        AffineNielsPoint::identity()
    }
}

// ------------------------------------------------------------------------
// Validity checks (for debugging, not CT)
// ------------------------------------------------------------------------

impl ValidityCheck for ProjectivePoint {
    fn is_valid(&self) -> bool {
        // Curve equation is    -x^2 + y^2 = 1 + d*x^2*y^2,
        // homogenized as (-X^2 + Y^2)*Z^2 = Z^4 + d*X^2*Y^2
        let XX = self.X.square();
        let YY = self.Y.square();
        let ZZ = self.Z.square();
        let ZZZZ = ZZ.square();
        let lhs = &(&YY - &XX) * &ZZ;
        let rhs = &ZZZZ + &(&constants::EDWARDS_D * &(&XX * &YY));

        lhs == rhs
    }
}

// ------------------------------------------------------------------------
// Constant-time assignment
// ------------------------------------------------------------------------

impl ConditionallySelectable for ProjectiveNielsPoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        ProjectiveNielsPoint {
            Y_plus_X: FieldElement::conditional_select(&a.Y_plus_X, &b.Y_plus_X, choice),
            Y_minus_X: FieldElement::conditional_select(&a.Y_minus_X, &b.Y_minus_X, choice),
            Z: FieldElement::conditional_select(&a.Z, &b.Z, choice),
            T2d: FieldElement::conditional_select(&a.T2d, &b.T2d, choice),
        }
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.Y_plus_X.conditional_assign(&other.Y_plus_X, choice);
        self.Y_minus_X.conditional_assign(&other.Y_minus_X, choice);
        self.Z.conditional_assign(&other.Z, choice);
        self.T2d.conditional_assign(&other.T2d, choice);
    }
}

impl ConditionallySelectable for AffineNielsPoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        AffineNielsPoint {
            y_plus_x: FieldElement::conditional_select(&a.y_plus_x, &b.y_plus_x, choice),
            y_minus_x: FieldElement::conditional_select(&a.y_minus_x, &b.y_minus_x, choice),
            xy2d: FieldElement::conditional_select(&a.xy2d, &b.xy2d, choice),
        }
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.y_plus_x.conditional_assign(&other.y_plus_x, choice);
        self.y_minus_x.conditional_assign(&other.y_minus_x, choice);
        self.xy2d.conditional_assign(&other.xy2d, choice);
    }
}

// ------------------------------------------------------------------------
// Point conversions
// ------------------------------------------------------------------------

impl ProjectivePoint {
    /// Convert this point from the \\( \mathbb P\^2 \\) model to the
    /// \\( \mathbb P\^3 \\) model.
    ///
    /// This costs \\(3 \mathrm M + 1 \mathrm S\\).
    pub fn to_extended(&self) -> EdwardsPoint {
        EdwardsPoint {
            X: &self.X * &self.Z,
            Y: &self.Y * &self.Z,
            Z: self.Z.square(),
            T: &self.X * &self.Y,
        }
    }
}

impl CompletedPoint {
    /// Convert this point from the \\( \mathbb P\^1 \times \mathbb P\^1
    /// \\) model to the \\( \mathbb P\^2 \\) model.
    ///
    /// This costs \\(3 \mathrm M \\).
    pub fn to_projective(&self) -> ProjectivePoint {
        ProjectivePoint {
            X: &self.X * &self.T,
            Y: &self.Y * &self.Z,
            Z: &self.Z * &self.T,
        }
    }

    /// Convert this point from the \\( \mathbb P\^1 \times \mathbb P\^1
    /// \\) model to the \\( \mathbb P\^3 \\) model.
    ///
    /// This costs \\(4 \mathrm M \\).
    pub fn to_extended(&self) -> EdwardsPoint {
        EdwardsPoint {
            X: &self.X * &self.T,
            Y: &self.Y * &self.Z,
            Z: &self.Z * &self.T,
            T: &self.X * &self.Y,
        }
    }
}

// ------------------------------------------------------------------------
// Doubling
// ------------------------------------------------------------------------

impl ProjectivePoint {
    /// Double this point: return self + self
    pub fn double(&self) -> CompletedPoint { // Double()
        let XX          = self.X.square();
        let YY          = self.Y.square();
        let ZZ2         = self.Z.square2();
        let X_plus_Y    = &self.X + &self.Y;
        let X_plus_Y_sq = X_plus_Y.square();
        let YY_plus_XX  = &YY + &XX;
        let YY_minus_XX = &YY - &XX;

        CompletedPoint{
            X: &X_plus_Y_sq - &YY_plus_XX,
            Y: YY_plus_XX,
            Z: YY_minus_XX,
            T: &ZZ2 - &YY_minus_XX
        }
    }
}

// ------------------------------------------------------------------------
// Addition and Subtraction
// ------------------------------------------------------------------------

// XXX(hdevalence) These were doc(hidden) so they don't appear in the
// public API docs.
// However, that prevents them being used with --document-private-items,
// so comment out the doc(hidden) for now until this is resolved
//
// upstream rust issue: https://github.com/rust-lang/rust/issues/46380
//#[doc(hidden)]
impl<'a, 'b> Add<&'b ProjectiveNielsPoint> for &'a EdwardsPoint {
    type Output = CompletedPoint;

    fn add(self, other: &'b ProjectiveNielsPoint) -> CompletedPoint {
        let Y_plus_X  = &self.Y + &self.X;
        let Y_minus_X = &self.Y - &self.X;
        let PP = &Y_plus_X  * &other.Y_plus_X;
        let MM = &Y_minus_X * &other.Y_minus_X;
        let TT2d = &self.T * &other.T2d;
        let ZZ   = &self.Z * &other.Z;
        let ZZ2  = &ZZ + &ZZ;

        CompletedPoint{
            X: &PP - &MM,
            Y: &PP + &MM,
            Z: &ZZ2 + &TT2d,
            T: &ZZ2 - &TT2d
        }
    }
}

//#[doc(hidden)]
impl<'a, 'b> Sub<&'b ProjectiveNielsPoint> for &'a EdwardsPoint {
    type Output = CompletedPoint;

    fn sub(self, other: &'b ProjectiveNielsPoint) -> CompletedPoint {
        let Y_plus_X  = &self.Y + &self.X;
        let Y_minus_X = &self.Y - &self.X;
        let PM = &Y_plus_X * &other.Y_minus_X;
        let MP = &Y_minus_X  * &other.Y_plus_X;
        let TT2d = &self.T * &other.T2d;
        let ZZ   = &self.Z * &other.Z;
        let ZZ2  = &ZZ + &ZZ;

        CompletedPoint{
            X: &PM - &MP,
            Y: &PM + &MP,
            Z: &ZZ2 - &TT2d,
            T: &ZZ2 + &TT2d
        }
    }
}

//#[doc(hidden)]
impl<'a, 'b> Add<&'b AffineNielsPoint> for &'a EdwardsPoint {
    type Output = CompletedPoint;

    fn add(self, other: &'b AffineNielsPoint) -> CompletedPoint {
        let Y_plus_X  = &self.Y + &self.X;
        let Y_minus_X = &self.Y - &self.X;
        let PP        = &Y_plus_X  * &other.y_plus_x;
        let MM        = &Y_minus_X * &other.y_minus_x;
        let Txy2d     = &self.T * &other.xy2d;
        let Z2        = &self.Z + &self.Z;

        CompletedPoint{
            X: &PP - &MM,
            Y: &PP + &MM,
            Z: &Z2 + &Txy2d,
            T: &Z2 - &Txy2d
        }
    }
}

//#[doc(hidden)]
impl<'a, 'b> Sub<&'b AffineNielsPoint> for &'a EdwardsPoint {
    type Output = CompletedPoint;

    fn sub(self, other: &'b AffineNielsPoint) -> CompletedPoint {
        let Y_plus_X  = &self.Y + &self.X;
        let Y_minus_X = &self.Y - &self.X;
        let PM        = &Y_plus_X  * &other.y_minus_x;
        let MP        = &Y_minus_X * &other.y_plus_x;
        let Txy2d     = &self.T * &other.xy2d;
        let Z2        = &self.Z + &self.Z;

        CompletedPoint{
            X: &PM - &MP,
            Y: &PM + &MP,
            Z: &Z2 - &Txy2d,
            T: &Z2 + &Txy2d
        }
    }
}

// ------------------------------------------------------------------------
// Negation
// ------------------------------------------------------------------------

impl<'a> Neg for &'a ProjectiveNielsPoint {
    type Output = ProjectiveNielsPoint;

    fn neg(self) -> ProjectiveNielsPoint {
        ProjectiveNielsPoint{
            Y_plus_X:   self.Y_minus_X,
            Y_minus_X:  self.Y_plus_X,
            Z:          self.Z,
            T2d:        -(&self.T2d),
        }
    }
}

impl<'a> Neg for &'a AffineNielsPoint {
    type Output = AffineNielsPoint;

    fn neg(self) -> AffineNielsPoint {
        AffineNielsPoint{
            y_plus_x:   self.y_minus_x,
            y_minus_x:  self.y_plus_x,
            xy2d:       -(&self.xy2d)
        }
    }
}

// ------------------------------------------------------------------------
// Debug traits
// ------------------------------------------------------------------------

impl Debug for ProjectivePoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "ProjectivePoint{{\n\tX: {:?},\n\tY: {:?},\n\tZ: {:?}\n}}",
               &self.X, &self.Y, &self.Z)
    }
}

impl Debug for CompletedPoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "CompletedPoint{{\n\tX: {:?},\n\tY: {:?},\n\tZ: {:?},\n\tT: {:?}\n}}",
               &self.X, &self.Y, &self.Z, &self.T)
    }
}

impl Debug for AffineNielsPoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "AffineNielsPoint{{\n\ty_plus_x: {:?},\n\ty_minus_x: {:?},\n\txy2d: {:?}\n}}",
               &self.y_plus_x, &self.y_minus_x, &self.xy2d)
    }
}

impl Debug for ProjectiveNielsPoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "ProjectiveNielsPoint{{\n\tY_plus_X: {:?},\n\tY_minus_X: {:?},\n\tZ: {:?},\n\tT2d: {:?}\n}}",
               &self.Y_plus_X, &self.Y_minus_X, &self.Z, &self.T2d)
    }
}


