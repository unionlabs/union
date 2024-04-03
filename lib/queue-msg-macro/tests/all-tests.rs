#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    // named-struct
    t.pass("tests/named-struct-no-type.rs");
    t.pass("tests/named-struct-one-type.rs");
    t.pass("tests/named-struct-multiple-types.rs");
    // unnamed-struct
    t.pass("tests/unnamed-struct.rs");
    t.pass("tests/unnamed-struct-one-type.rs");
    t.compile_fail("tests/unnamed-struct-no-type.rs");
    t.compile_fail("tests/unnamed-struct-multiple-types.rs");
    // enum
    t.pass("tests/enum.rs");
    // unsupported
    t.compile_fail("tests/union.rs");
    t.compile_fail("tests/non-derive.rs");
    t.compile_fail("tests/unit-struct.rs");
}
