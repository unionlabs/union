// Simple tests using MIRI. These are intended only to be a simple exercise of
// memchr when tests are run under miri. These are mostly necessary because the
// other tests are far more extensive and take too long to run under miri.
//
// These tests are also run when the 'std' feature is not enabled.

use crate::{memchr, memchr2, memchr3, memrchr, memrchr2, memrchr3};

#[test]
fn simple() {
    assert_eq!(memchr(b'a', b"abcda"), Some(0));
    assert_eq!(memchr(b'z', b"abcda"), None);
    assert_eq!(memchr2(b'a', b'z', b"abcda"), Some(0));
    assert_eq!(memchr2(b'z', b'y', b"abcda"), None);
    assert_eq!(memchr3(b'a', b'z', b'b', b"abcda"), Some(0));
    assert_eq!(memchr3(b'z', b'y', b'x', b"abcda"), None);
    assert_eq!(memrchr(b'a', b"abcda"), Some(4));
    assert_eq!(memrchr(b'z', b"abcda"), None);
    assert_eq!(memrchr2(b'a', b'z', b"abcda"), Some(4));
    assert_eq!(memrchr2(b'z', b'y', b"abcda"), None);
    assert_eq!(memrchr3(b'a', b'z', b'b', b"abcda"), Some(4));
    assert_eq!(memrchr3(b'z', b'y', b'x', b"abcda"), None);
}
