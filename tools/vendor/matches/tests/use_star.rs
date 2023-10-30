//! https://github.com/SimonSapin/rust-std-candidates/issues/22

extern crate matches;

use matches::*;

#[test]
fn test_assert_matches() {
    assert_matches!(4, 4)
}
