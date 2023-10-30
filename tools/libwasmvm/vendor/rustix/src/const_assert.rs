/// A simple `assert` macro that works in `const fn`, for use until the
/// standard `assert` macro works in `const fn`.
///
/// TODO: With Rust 1.58 we can replace this with just `assert!`.
#[allow(unused_macros)]
macro_rules! const_assert {
    ($x:expr) => {
        let b: bool = $x;
        let _ = [()][!b as usize];
    };
}

#[test]
#[allow(clippy::missing_const_for_fn)]
fn test_const_assert() {
    const_assert!(true);
}

#[test]
const fn test_const_assert_in_const_fn() {
    const_assert!(true);
}
