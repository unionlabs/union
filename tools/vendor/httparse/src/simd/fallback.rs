use crate::iter::Bytes;

// Fallbacks that do nothing...

#[inline(always)]
pub fn match_uri_vectored(_: &mut Bytes<'_>) {}
#[inline(always)]
pub fn match_header_value_vectored(_: &mut Bytes<'_>) {}
