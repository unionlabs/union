#![cfg(test)]

use cosmwasm_std::{Decimal, Uint128};
use std::str::FromStr as _;

/// Asserts that two expressions are approximately equal to each other.
///
/// The `max_rel_diff` argument defines the maximum relative difference
/// of the `left` and `right` values.
///
/// On panic, this macro will print the values of the arguments and
/// the actual relative difference.
///
/// Like [`assert_eq!`], this macro has a second form, where a custom
/// panic message can be provided.
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $max_rel_diff:expr $(,)?) => {{
        $crate::test_utils::assert_approx_eq_impl($left, $right, $max_rel_diff, None);
    }};
    ($left:expr, $right:expr, $max_rel_diff:expr, $($args:tt)+) => {{
        $crate::test_utils::assert_approx_eq_impl($left, $right, $max_rel_diff, Some(format!($($args)*)));
    }};
}

#[track_caller]
pub fn assert_approx_eq_impl<U: Into<Uint128>>(
    left: U,
    right: U,
    max_rel_diff: &str,
    panic_msg: Option<String>,
) {
    let left = left.into();
    let right = right.into();
    let max_rel_diff = Decimal::from_str(max_rel_diff).unwrap();

    let largest = std::cmp::max(left, right);
    let rel_diff = Decimal::from_ratio(left.abs_diff(right), largest);

    if rel_diff > max_rel_diff {
        match panic_msg {
            Some(panic_msg) => panic!(
                "assertion failed: `(left ≈ right)`\nleft: {left}\nright: {right}\nrelative difference: {rel_diff}\nmax allowed relative difference: {max_rel_diff}\n: {panic_msg}"
            ),
            None => panic!(
                "assertion failed: `(left ≈ right)`\nleft: {left}\nright: {right}\nrelative difference: {rel_diff}\nmax allowed relative difference: {max_rel_diff}\n"
            ),
        }
    }
}

mod tests {
    #[test]
    fn assert_approx() {
        assert_approx_eq!(9_u32, 10_u32, "0.12");
        assert_approx_eq!(9_u64, 10_u64, "0.12");
        assert_approx_eq!(
            9_000_000_000_000_000_000_000_000_000_000_000_000_u128,
            10_000_000_000_000_000_000_000_000_000_000_000_000_u128,
            "0.10"
        );
    }

    #[test]
    fn assert_approx_with_vars() {
        let a = 66_u32;
        let b = 67_u32;
        assert_approx_eq!(a, b, "0.02");

        let a = 66_u64;
        let b = 67_u64;
        assert_approx_eq!(a, b, "0.02");

        let a = 66_u128;
        let b = 67_u128;
        assert_approx_eq!(a, b, "0.02");
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: `(left ≈ right)`\nleft: 8\nright: 10\nrelative difference: 0.2\nmax allowed relative difference: 0.12\n"
    )]
    fn assert_approx_fail() {
        assert_approx_eq!(8_u32, 10_u32, "0.12");
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: `(left ≈ right)`\nleft: 17\nright: 20\nrelative difference: 0.15\nmax allowed relative difference: 0.12\n: some extra info about the error"
    )]
    fn assert_approx_with_custom_panic_msg() {
        assert_approx_eq!(
            17_u32,
            20_u32,
            "0.12",
            "some extra {} about the error",
            "info"
        );
    }
}
