#![allow(dead_code)]

struct Foo {
    secret: u64,
}

opaque_debug::implement!(Foo);

#[test]
fn debug_formatting() {
    let s = format!("{:?}", Foo { secret: 42 });
    assert_eq!(s, "Foo { ... }");
}
