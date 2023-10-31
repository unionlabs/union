use crate::util::is_continuation;

use super::encode_wide;

const SURROGATE_LENGTH: usize = 3;

pub(crate) fn ends_with(string: &[u8], mut suffix: &[u8]) -> bool {
    let index = match string.len().checked_sub(suffix.len()) {
        Some(index) => index,
        None => return false,
    };
    if let Some(&byte) = string.get(index) {
        if is_continuation(byte) {
            let index = index.checked_sub(1).expect("invalid string");
            let mut wide_surrogate = match suffix.get(..SURROGATE_LENGTH) {
                Some(surrogate) => encode_wide(surrogate),
                None => return false,
            };
            let surrogate_wchar = wide_surrogate
                .next()
                .expect("failed decoding non-empty suffix");

            if wide_surrogate.next().is_some()
                || encode_wide(&string[index..])
                    .take_while(Result::is_ok)
                    .nth(1)
                    != Some(surrogate_wchar)
            {
                return false;
            }
            suffix = &suffix[SURROGATE_LENGTH..];
        }
    }
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], mut prefix: &[u8]) -> bool {
    if let Some(&byte) = string.get(prefix.len()) {
        if is_continuation(byte) {
            let index = match prefix.len().checked_sub(SURROGATE_LENGTH) {
                Some(index) => index,
                None => return false,
            };
            let (substring, surrogate) = prefix.split_at(index);
            let mut wide_surrogate = encode_wide(surrogate);
            let surrogate_wchar = wide_surrogate
                .next()
                .expect("failed decoding non-empty prefix");

            if surrogate_wchar.is_err()
                || wide_surrogate.next().is_some()
                || encode_wide(&string[index..])
                    .next()
                    .expect("failed decoding non-empty substring")
                    != surrogate_wchar
            {
                return false;
            }
            prefix = substring;
        }
    }
    string.starts_with(prefix)
}
