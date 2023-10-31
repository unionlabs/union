/// A simple `assert` macro that works in `const fn`, for use until the
/// standard `assert` macro works in `const fn`.
///
/// TODO: Replace this with just `assert!`, once that's stable for use in
/// a `const fn` context.
#[allow(unused_macros)]
macro_rules! const_assert {
    ($x:expr) => {
        let b: bool = $x;
        let _ = [()][!b as usize];
    };
}

#[test]
fn test_const_assert() {
    const_assert!(true);
}

#[test]
const fn test_const_assert_in_const_fn() {
    const_assert!(true);
}
