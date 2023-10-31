use std::fmt;
use std::fmt::Formatter;

use super::Result;

#[inline(always)]
pub(crate) const fn is_continuation(_: u8) -> bool {
    false
}

#[inline(always)]
pub(crate) fn validate_bytes(_: &[u8]) -> Result<()> {
    Ok(())
}

#[inline(always)]
pub(crate) fn decode_code_point(_: &[u8]) -> u32 {
    unreachable!();
}

pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
    string.starts_with(prefix)
}

pub(crate) fn debug(string: &[u8], f: &mut Formatter<'_>) -> fmt::Result {
    for byte in string {
        write!(f, "\\x{:02X}", byte)?;
    }
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
