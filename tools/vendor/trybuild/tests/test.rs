#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/run-pass-0.rs");
    t.pass("tests/ui/print-stdout.rs");
    t.pass("tests/ui/run-pass-1.rs");
    t.pass("tests/ui/print-stderr.rs");
    t.pass("tests/ui/run-pass-2.rs");
    t.pass("tests/ui/print-both.rs");
    t.pass("tests/ui/run-pass-4.rs");
    t.compile_fail("tests/ui/run-pass-3.rs");
    t.pass("tests/ui/run-pass-5.rs");
    t.pass("tests/ui/compile-fail-0.rs");
    t.pass("tests/ui/run-pass-6.rs");
    t.pass("tests/ui/run-pass-7.rs");
    t.pass("tests/ui/run-pass-8.rs");
    t.compile_fail("tests/ui/compile-fail-1.rs");
    t.pass("tests/ui/run-fail.rs");
    t.pass("tests/ui/run-pass-9.rs");
    t.compile_fail("tests/ui/compile-fail-2.rs");
    t.compile_fail("tests/ui/compile-fail-3.rs");
}
