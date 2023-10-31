#![allow(non_snake_case)]

use backend::vector::{CachedPoint, ExtendedPoint};
use edwards::EdwardsPoint;
use scalar::Scalar;
use traits::Identity;
use window::LookupTable;

/// Perform constant-time, variable-base scalar multiplication.
pub fn mul(point: &EdwardsPoint, scalar: &Scalar) -> EdwardsPoint {
    // Construct a lookup table of [P,2P,3P,4P,5P,6P,7P,8P]
    let lookup_table = LookupTable::<CachedPoint>::from(point);
    // Setting s = scalar, compute
    //
    //    s = s_0 + s_1*16^1 + ... + s_63*16^63,
    //
    // with `-8 ≤ s_i < 8` for `0 ≤ i < 63` and `-8 ≤ s_63 ≤ 8`.
    let scalar_digits = scalar.to_radix_16();
    // Compute s*P as
    //
    //    s*P = P*(s_0 +   s_1*16^1 +   s_2*16^2 + ... +   s_63*16^63)
    //    s*P =  P*s_0 + P*s_1*16^1 + P*s_2*16^2 + ... + P*s_63*16^63
    //    s*P = P*s_0 + 16*(P*s_1 + 16*(P*s_2 + 16*( ... + P*s_63)...))
    //
    // We sum right-to-left.
    let mut Q = ExtendedPoint::identity();
    for i in (0..64).rev() {
        Q = Q.mul_by_pow_2(4);
        Q = &Q + &lookup_table.select(scalar_digits[i]);
    }
    Q.into()
}
