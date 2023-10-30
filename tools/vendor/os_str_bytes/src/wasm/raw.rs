use std::fmt;
use std::fmt::Formatter;
use std::str;

pub(crate) use crate::util::is_continuation;

use super::Result;

#[allow(dead_code)]
#[path = "../common/raw.rs"]
mod common_raw;
pub(crate) use common_raw::ends_with;
pub(crate) use common_raw::starts_with;
#[cfg(feature = "uniquote")]
pub(crate) use common_raw::uniquote;

pub(crate) fn validate_bytes(string: &[u8]) -> Result<()> {
    super::from_bytes(string).map(drop)
}

pub(crate) fn decode_code_point(string: &[u8]) -> u32 {
    let string = expect_encoded!(str::from_utf8(string));
    let mut chars = string.chars();
    let ch = chars
        .next()
        .expect("cannot parse code point from empty string");
    assert_eq!(None, chars.next(), "multiple code points found");
    ch.into()
}

pub(crate) fn debug(string: &[u8], _: &mut Formatter<'_>) -> fmt::Result {
    assert!(string.is_empty());
    Ok(())
}
