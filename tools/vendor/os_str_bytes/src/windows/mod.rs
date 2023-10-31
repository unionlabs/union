// These functions are necessarily inefficient, because they must revert
// encoding conversions performed by the standard library. However, there is
// currently no better alternative.

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Not;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::result;
use std::str;

if_raw_str! {
    pub(super) mod raw;
}

mod wtf8;
use wtf8::DecodeWide;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EncodingError {
    Byte(u8),
    CodePoint(u32),
    End(),
}

impl EncodingError {
    fn position(&self) -> Cow<'_, str> {
        match self {
            Self::Byte(byte) => Cow::Owned(format!("byte b'\\x{:02X}'", byte)),
            Self::CodePoint(code_point) => {
                Cow::Owned(format!("code point U+{:04X}", code_point))
            }
            Self::End() => Cow::Borrowed("end of string"),
        }
    }
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "byte sequence is not representable in the platform encoding; \
             error at {}",
            self.position(),
        )
    }
}

impl Error for EncodingError {}

type Result<T> = result::Result<T, EncodingError>;

fn from_bytes(string: &[u8]) -> Result<Option<OsString>> {
    let mut encoder = wtf8::encode_wide(string);

    // Collecting an iterator into a result ignores the size hint:
    // https://github.com/rust-lang/rust/issues/48994
    let mut encoded_string = Vec::with_capacity(encoder.size_hint().0);
    for wchar in &mut encoder {
        encoded_string.push(wchar?);
    }

    debug_assert_eq!(str::from_utf8(string).is_ok(), encoder.is_still_utf8());
    Ok(encoder
        .is_still_utf8()
        .not()
        .then(|| OsStringExt::from_wide(&encoded_string)))
}

fn to_bytes(os_string: &OsStr) -> Vec<u8> {
    let encoder = OsStrExt::encode_wide(os_string);

    let mut string = Vec::with_capacity(encoder.size_hint().0);
    string.extend(DecodeWide::new(encoder));
    string
}

pub(super) fn os_str_from_bytes(string: &[u8]) -> Result<Cow<'_, OsStr>> {
    from_bytes(string).map(|os_string| {
        os_string.map(Cow::Owned).unwrap_or_else(|| {
            // SAFETY: This slice was validated to be UTF-8.
            Cow::Borrowed(OsStr::new(unsafe {
                str::from_utf8_unchecked(string)
            }))
        })
    })
}

pub(super) fn os_str_to_bytes(os_string: &OsStr) -> Cow<'_, [u8]> {
    Cow::Owned(to_bytes(os_string))
}

pub(super) fn os_string_from_vec(string: Vec<u8>) -> Result<OsString> {
    from_bytes(&string).map(|os_string| {
        os_string.unwrap_or_else(|| {
            // SAFETY: This slice was validated to be UTF-8.
            unsafe { String::from_utf8_unchecked(string) }.into()
        })
    })
}

pub(super) fn os_string_into_vec(os_string: OsString) -> Vec<u8> {
    to_bytes(&os_string)
}
