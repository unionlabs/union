//! Optimized simplified Shallue-van de Woestijne-Ulas methods.
//!
//! <https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#straightline-sswu>

use ff::Field;
use subtle::Choice;
use subtle::ConditionallySelectable;
use subtle::ConstantTimeEq;

/// The Optimized Simplified Shallue-van de Woestijne-Ulas parameters
pub struct OsswuMapParams<F>
where
    F: Field,
{
    /// The first constant term
    pub c1: &'static [u64],
    /// The second constant term
    pub c2: F,
    /// The ISO A variable or Curve A variable
    pub map_a: F,
    /// The ISO A variable or Curve A variable
    pub map_b: F,
    /// The Z parameter
    pub z: F,
}

/// Trait for determining the parity of the field
pub trait Sgn0 {
    /// Return the parity of the field
    /// 1 == negative
    /// 0 == non-negative
    fn sgn0(&self) -> Choice;
}

/// The optimized simplified Shallue-van de Woestijne-Ulas method
/// for mapping elliptic curve scalars to affine points.
pub trait OsswuMap: Field + Sgn0 {
    /// The OSSWU parameters for mapping the field to affine points.
    /// For Weierstrass curves having A==0 or B==0, the parameters
    /// should be for isogeny where A≠0 and B≠0.
    const PARAMS: OsswuMapParams<Self>;

    /// Optimized sqrt_ratio for q = 3 mod 4.
    fn sqrt_ratio_3mod4(u: Self, v: Self) -> (Choice, Self) {
        // 1. tv1 = v^2
        let tv1 = v.square();
        // 2. tv2 = u * v
        let tv2 = u * v;
        // 3. tv1 = tv1 * tv2
        let tv1 = tv1 * tv2;
        // 4. y1 = tv1^c1
        let y1 = tv1.pow_vartime(Self::PARAMS.c1);
        // 5. y1 = y1 * tv2
        let y1 = y1 * tv2;
        // 6. y2 = y1 * c2
        let y2 = y1 * Self::PARAMS.c2;
        // 7. tv3 = y1^2
        let tv3 = y1.square();
        // 8. tv3 = tv3 * v
        let tv3 = tv3 * v;
        // 9. isQR = tv3 == u
        let is_qr = tv3.ct_eq(&u);
        // 10. y = CMOV(y2, y1, isQR)
        let y = ConditionallySelectable::conditional_select(&y2, &y1, is_qr);
        // 11. return (isQR, y)
        (is_qr, y)
    }

    /// Convert this field element into an affine point on the elliptic curve
    /// returning (X, Y). For Weierstrass curves having A==0 or B==0
    /// the result is a point on an isogeny.
    fn osswu(&self) -> (Self, Self) {
        // 1.  tv1 = u^2
        let tv1 = self.square();
        // 2.  tv1 = Z * tv1
        let tv1 = Self::PARAMS.z * tv1;
        // 3.  tv2 = tv1^2
        let tv2 = tv1.square();
        // 4.  tv2 = tv2 + tv1
        let tv2 = tv2 + tv1;
        // 5.  tv3 = tv2 + 1
        let tv3 = tv2 + Self::ONE;
        // 6.  tv3 = B * tv3
        let tv3 = Self::PARAMS.map_b * tv3;
        // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
        let tv4 = ConditionallySelectable::conditional_select(
            &Self::PARAMS.z,
            &-tv2,
            !Field::is_zero(&tv2),
        );
        // 8.  tv4 = A * tv4
        let tv4 = Self::PARAMS.map_a * tv4;
        // 9.  tv2 = tv3^2
        let tv2 = tv3.square();
        // 10. tv6 = tv4^2
        let tv6 = tv4.square();
        // 11. tv5 = A * tv6
        let tv5 = Self::PARAMS.map_a * tv6;
        // 12. tv2 = tv2 + tv5
        let tv2 = tv2 + tv5;
        // 13. tv2 = tv2 * tv3
        let tv2 = tv2 * tv3;
        // 14. tv6 = tv6 * tv4
        let tv6 = tv6 * tv4;
        // 15. tv5 = B * tv6
        let tv5 = Self::PARAMS.map_b * tv6;
        // 16. tv2 = tv2 + tv5
        let tv2 = tv2 + tv5;
        // 17.   x = tv1 * tv3
        let x = tv1 * tv3;
        // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
        let (is_gx1_square, y1) = Self::sqrt_ratio_3mod4(tv2, tv6);
        // 19.   y = tv1 * u
        let y = tv1 * self;
        // 20.   y = y * y1
        let y = y * y1;
        // 21.   x = CMOV(x, tv3, is_gx1_square)
        let x = ConditionallySelectable::conditional_select(&x, &tv3, is_gx1_square);
        // 22.   y = CMOV(y, y1, is_gx1_square)
        let y = ConditionallySelectable::conditional_select(&y, &y1, is_gx1_square);
        // 23.  e1 = sgn0(u) == sgn0(y)
        let e1 = self.sgn0().ct_eq(&y.sgn0());
        // 24.   y = CMOV(-y, y, e1)
        let y = ConditionallySelectable::conditional_select(&-y, &y, e1);
        // 25.   x = x / tv4
        let x = x * tv4.invert().unwrap();
        // 26. return (x, y)
        (x, y)
    }
}
