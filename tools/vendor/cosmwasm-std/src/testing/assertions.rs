use crate::{Decimal, Uint128};
#[cfg(test)]
use core::hash::{Hash, Hasher};
use core::str::FromStr as _;

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
        $crate::testing::assert_approx_eq_impl($left, $right, $max_rel_diff, None);
    }};
    ($left:expr, $right:expr, $max_rel_diff:expr, $($args:tt)+) => {{
        $crate::testing::assert_approx_eq_impl($left, $right, $max_rel_diff, Some(format!($($args)*)));
    }};
}

/// Tests that type `T` implements `Eq` and `Hash` traits correctly.
///
/// `left` and `right` must be unequal objects.
///
/// Some object pairs may produce the same hash causing test failure.
/// In those cases try different objects. The test uses stable hasher
/// so once working pair is identified, the test’s going to continue
/// passing.
#[macro_export]
#[cfg(test)]
macro_rules! assert_hash_works {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::testing::assert_hash_works_impl($left, $right, None);
    }};
    ($left:expr, $right:expr, $($args:tt)+) => {{
        $crate::testing::assert_hash_works_impl($left, $right, Some(format!($($args)*)));
    }};
}

/// Implementation for the [`cosmwasm_std::assert_approx_eq`] macro. This does not provide any
/// stability guarantees and may change any time.
#[track_caller]
#[doc(hidden)]
pub fn assert_approx_eq_impl<U: Into<Uint128>>(
    left: U,
    right: U,
    max_rel_diff: &str,
    panic_msg: Option<String>,
) {
    let left = left.into();
    let right = right.into();
    let max_rel_diff = Decimal::from_str(max_rel_diff).unwrap();

    let largest = core::cmp::max(left, right);
    let rel_diff = Decimal::from_ratio(left.abs_diff(right), largest);

    if rel_diff > max_rel_diff {
        do_panic(format_args!("assertion failed: `(left ≈ right)`\nleft: {left}\nright: {right}\nrelative difference: {rel_diff}\nmax allowed relative difference: {max_rel_diff}"), panic_msg);
    }
}

/// Tests that type `T` implements `Eq` and `Hash` traits correctly.
///
/// `left` and `right` must be unequal objects.
///
/// Some object pairs may produce the same hash causing test failure.  In those
/// cases try different objects. The test uses stable hasher so once working
/// pair is identified, the test’s going to continue passing.
#[track_caller]
#[doc(hidden)]
#[cfg(test)]
pub fn assert_hash_works_impl<T: Clone + Eq + Hash>(left: T, right: T, panic_msg: Option<String>) {
    fn hash(value: &impl Hash) -> u64 {
        let mut hasher = crc32fast::Hasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    // Check clone
    #[allow(clippy::redundant_clone)]
    let clone = left.clone();
    if left != clone {
        do_panic("assertion failed: `left == left.clone()`", panic_msg);
    }
    if hash(&left) != hash(&clone) {
        do_panic(
            "assertion failed: `hash(left) == hash(left.clone())`",
            panic_msg,
        );
    }

    // Check different object
    if left == right {
        do_panic("assertion failed: `left != right`", panic_msg);
    }
    if hash(&left) == hash(&right) {
        do_panic("assertion failed: `hash(left) != hash(right)`", panic_msg);
    }
}

/// Panics concatenating both arguments.
///
/// If second argument is `None` panics with just the first argument as message.
/// Otherwise, formats the panic message as `{reason}:\n{panic_msg}`.
#[track_caller]
fn do_panic(reason: impl core::fmt::Display, panic_msg: Option<String>) -> ! {
    match panic_msg {
        Some(panic_msg) => panic!("{reason}:\n{panic_msg}"),
        None => panic!("{reason}"),
    }
}

#[cfg(test)]
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
        expected = "assertion failed: `(left ≈ right)`\nleft: 8\nright: 10\nrelative difference: 0.2\nmax allowed relative difference: 0.12"
    )]
    fn assert_approx_fail() {
        assert_approx_eq!(8_u32, 10_u32, "0.12");
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: `(left ≈ right)`\nleft: 17\nright: 20\nrelative difference: 0.15\nmax allowed relative difference: 0.12:\nsome extra info about the error: Foo(8)"
    )]
    fn assert_approx_with_custom_panic_msg() {
        let adjective = "extra";
        #[derive(Debug)]
        struct Foo(u32);
        assert_approx_eq!(
            17_u32,
            20_u32,
            "0.12",
            "some {adjective} {} about the error: {:?}",
            "info",
            Foo(8),
        );
    }
}
