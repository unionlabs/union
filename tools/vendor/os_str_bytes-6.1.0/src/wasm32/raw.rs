use std::fmt;
use std::fmt::Formatter;
use std::str;

pub(crate) use crate::util::is_continuation;

pub(crate) fn decode_code_point(string: &[u8]) -> u32 {
    let string = str::from_utf8(string).expect("invalid string");
    let mut chars = string.chars();
    let ch = chars
        .next()
        .expect("cannot parse code point from empty string");
    assert_eq!(None, chars.next(), "multiple code points found");
    ch.into()
}

pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
    string.starts_with(prefix)
}

pub(crate) fn debug(string: &[u8], _: &mut Formatter<'_>) -> fmt::Result {
    assert!(string.is_empty());
    Ok(())
}

#[cfg(feature = "uniquote")]
pub(crate) mod uniquote {
    use uniquote::Formatter;
    use uniquote::Quote;
    use uniquote::Result;

    pub(crate) fn escape(string: &[u8], f: &mut Formatter<'_>) -> Result {
        string.escape(f)
    }
}
