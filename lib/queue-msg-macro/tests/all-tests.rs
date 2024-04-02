#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/named-struct-no-type.rs");
    t.pass("tests/named-struct-one-type.rs");
    t.pass("tests/named-struct-multiple-types.rs");
    t.compile_fail("tests/unnamed-struct-no-type.rs");
    t.pass("tests/unnamed-struct-one-type.rs");
    t.pass("tests/enum.rs");
    t.compile_fail("tests/unnamed-struct-multiple-types.rs");
    t.compile_fail("tests/union.rs");
    t.compile_fail("tests/non-derive.rs");
    t.compile_fail("tests/unit-struct.rs");
}
