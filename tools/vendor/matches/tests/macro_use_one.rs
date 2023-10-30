// https://github.com/SimonSapin/rust-std-candidates/issues/12
#[macro_use(matches)] extern crate matches;

#[test]
fn matches_works() {
    let foo = Some("-12");
    assert!(matches!(foo, Some(bar) if
        matches!(bar.as_bytes()[0], b'+' | b'-') &&
        matches!(bar.as_bytes()[1], b'0'...b'9')
    ));
}
