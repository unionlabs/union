#[cfg(feature = "macro-diagnostics")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/compile_pass/*.rs");

    if rustversion::cfg!(nightly) {
        t.compile_fail("tests/ui/compile_fail/*.rs");
    }
}
