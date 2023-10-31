mod support;

use crate::support::Test;
use std::env;

/// This test is in its own module because it modifies the environment and would affect other tests
/// when run in parallel with them.
#[test]
fn gnu_no_warnings_if_cxxflags() {
    env::set_var("CXXFLAGS", "-arbitrary");
    let test = Test::gnu();
    test.gcc().file("foo.cpp").cpp(true).compile("foo");

    test.cmd(0).must_not_have("-Wall").must_not_have("-Wextra");
}
