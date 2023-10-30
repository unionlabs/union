#![cfg(not(miri))]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use std::env;

#[rustversion::attr(not(nightly), ignore)]
#[test]
fn ui() {
    if env::var_os("CI").is_none() {
        env::set_var("TRYBUILD", "overwrite");
    }

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
    t.pass("tests/run-pass/**/*.rs");
}
