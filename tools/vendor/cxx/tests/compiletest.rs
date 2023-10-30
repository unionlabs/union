#[allow(unused_attributes)]
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(skip_ui_tests, ignore)]
#[cfg_attr(miri, ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
