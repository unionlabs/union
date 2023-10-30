use std::{cmp, ops};

use crate::{Uint128, Uint256, Uint512, Uint64};

/// A trait for calculating the
/// [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
pub trait Isqrt {
    /// The [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn isqrt(self) -> Self;
}

impl<I> Isqrt for I
where
    I: Unsigned
        + ops::Add<I, Output = I>
        + ops::Div<I, Output = I>
        + ops::Shr<u32, Output = I>
        + cmp::PartialOrd
        + Copy
        + From<u8>,
{
    /// Algorithm adapted from
    /// [Wikipedia](https://en.wikipedia.org/wiki/Integer_square_root#Example_implementation_in_C).
    fn isqrt(self) -> Self {
        let mut x0 = self >> 1;

        if x0 > 0.into() {
            let mut x1 = (x0 + self / x0) >> 1;

            while x1 < x0 {
                x0 = x1;
                x1 = (x0 + self / x0) >> 1;
            }

            return x0;
        }
        self
    }
}

/// Marker trait for types that represent unsigned integers.
pub trait Unsigned {}
impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}
impl Unsigned for Uint64 {}
impl Unsigned for Uint128 {}
impl Unsigned for Uint256 {}
impl Unsigned for Uint512 {}
impl Unsigned for usize {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isqrt_primitives() {
        // Let's check correctness.
        assert_eq!(0u8.isqrt(), 0);
        assert_eq!(1u8.isqrt(), 1);
        assert_eq!(24u8.isqrt(), 4);
        assert_eq!(25u8.isqrt(), 5);
        assert_eq!(26u8.isqrt(), 5);
        assert_eq!(36u8.isqrt(), 6);

        // Let's also check different types.
        assert_eq!(26u8.isqrt(), 5);
        assert_eq!(26u16.isqrt(), 5);
        assert_eq!(26u32.isqrt(), 5);
        assert_eq!(26u64.isqrt(), 5);
        assert_eq!(26u128.isqrt(), 5);
    }

    #[test]
    fn isqrt_uint64() {
        assert_eq!(Uint64::new(24).isqrt(), Uint64::new(4));
    }

    #[test]
    fn isqrt_uint128() {
        assert_eq!(Uint128::new(24).isqrt(), Uint128::new(4));
    }

    #[test]
    fn isqrt_uint256() {
        assert_eq!(Uint256::from(24u32).isqrt(), Uint256::from(4u32));
        assert_eq!(
            (Uint256::from(u128::MAX) * Uint256::from(u128::MAX)).isqrt(),
            Uint256::try_from("340282366920938463463374607431768211455").unwrap()
        );
    }

    #[test]
    fn isqrt_uint512() {
        assert_eq!(Uint512::from(24u32).isqrt(), Uint512::from(4u32));
        assert_eq!(
            (Uint512::from(Uint256::MAX) * Uint512::from(Uint256::MAX)).isqrt(),
            Uint512::try_from(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
    }
}
