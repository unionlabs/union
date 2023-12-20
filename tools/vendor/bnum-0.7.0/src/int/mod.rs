pub mod cast;
pub mod checked;
pub mod cmp;
pub mod endian;
pub mod fmt;

#[cfg(feature = "numtraits")]
pub mod numtraits;

pub mod ops;
pub mod radix;
pub mod unchecked;

#[cfg(test)]
macro_rules! tests {
    ($int: ty) => {
        test_bignum! {
            function: <$int>::count_ones(a: $int)
        }
        test_bignum! {
            function: <$int>::count_zeros(a: $int)
        }
        test_bignum! {
            function: <$int>::leading_zeros(a: $int)
        }
        test_bignum! {
            function: <$int>::trailing_zeros(a: $int)
        }
        test_bignum! {
            function: <$int>::leading_ones(a: $int)
        }
        test_bignum! {
            function: <$int>::trailing_ones(a: $int)
        }
        test_bignum! {
            function: <$int>::rotate_left(a: $int, b: u8)
        }
        test_bignum! {
            function: <$int>::rotate_right(a: $int, b: u8)
        }
        test_bignum! {
            function: <$int>::swap_bytes(a: $int)
        }
        test_bignum! {
            function: <$int>::reverse_bits(a: $int)
        }
        test_bignum! {
            function: <$int>::pow(a: $int, b: u16),
            skip: crate::test::debug_skip!(a.checked_pow(b as u32).is_none())
        }
        test_bignum! {
            function: <$int>::div_euclid(a: $int, b: $int),
            skip: a.checked_div(b).is_none()
        }
        test_bignum! {
            function: <$int>::rem_euclid(a: $int, b: $int),
            skip: a.checked_rem(b).is_none()
        }
        test_bignum! {
            function: <$int>::abs_diff(a: $int, b: $int)
        }
        test_bignum! {
            function: <$int>::ilog(a: $int, base: $int),
            skip: crate::test::debug_skip!(a <= 0 || base <= 1)
        }
        test_bignum! {
            function: <$int>::ilog2(a: $int),
            skip: crate::test::debug_skip!(a <= 0)
        }
        test_bignum! {
            function: <$int>::ilog10(a: $int),
            skip: crate::test::debug_skip!(a <= 0)
        }
        test_bignum! {
            function: <$int>::checked_next_multiple_of(a: $int, b: $int)
        }
        test_bignum! {
            function: <$int>::next_multiple_of(a: $int, b: $int),
            skip: crate::test::debug_skip!(a.checked_next_multiple_of(b).is_none())
        }
        test_bignum! {
            function: <$int>::div_floor(a: $int, b: $int),
            skip: a.checked_div(b).is_none()
        }
        test_bignum! {
            function: <$int>::div_ceil(a: $int, b: $int),
            skip: a.checked_div(b).is_none()
        }
    };
}

#[cfg(test)]
pub(crate) use tests;
