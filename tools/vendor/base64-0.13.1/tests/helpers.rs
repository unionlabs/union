extern crate base64;

use base64::*;

pub fn compare_decode(expected: &str, target: &str) {
    assert_eq!(
        expected,
        String::from_utf8(decode(target).unwrap()).unwrap()
    );
    assert_eq!(
        expected,
        String::from_utf8(decode(target.as_bytes()).unwrap()).unwrap()
    );
}
