use std::fmt;
use std::fmt::Formatter;

pub(crate) use crate::util::is_continuation;

use super::wtf8;
pub(crate) use super::wtf8::ends_with;
pub(crate) use super::wtf8::starts_with;
use super::wtf8::CodePoints;
use super::Result;

pub(crate) fn validate_bytes(string: &[u8]) -> Result<()> {
    wtf8::encode_wide(string).try_for_each(|x| x.map(drop))
}

pub(crate) fn encode_wide_unchecked(
    string: &[u8],
) -> impl '_ + Iterator<Item = u16> {
    wtf8::encode_wide(string).map(|x| expect_encoded!(x))
}

pub(crate) fn decode_code_point(string: &[u8]) -> u32 {
    let mut code_points = CodePoints::new(string.iter().copied());
    let code_point = expect_encoded!(code_points
        .next()
        .expect("cannot parse code point from empty string"));
    assert_eq!(None, code_points.next(), "multiple code points found");
    code_point
}

pub(crate) fn debug(string: &[u8], f: &mut Formatter<'_>) -> fmt::Result {
    for wchar in encode_wide_unchecked(string) {
        write!(f, "\\u{{{:X}}}", wchar)?;
    }
    Ok(())
}

#[cfg(feature = "uniquote")]
pub(crate) mod uniquote {
    use uniquote::Formatter;
    use uniquote::Result;

    pub(crate) fn escape(string: &[u8], f: &mut Formatter<'_>) -> Result {
        f.escape_utf16(super::encode_wide_unchecked(string))
    }
}
