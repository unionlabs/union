// This module implements the WTF-8 encoding specification:
// https://simonsapin.github.io/wtf-8/

use super::EncodingError;
use super::Result;

mod code_points;
pub(super) use code_points::CodePoints;

mod convert;
pub(super) use convert::encode_wide;
pub(super) use convert::DecodeWide;

if_raw_str! {
    mod string;
    pub(crate) use string::ends_with;
    pub(crate) use string::starts_with;
}
